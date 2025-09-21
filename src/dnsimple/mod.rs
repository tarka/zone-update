// FIXME
#![allow(unused)]

mod types;

use std::{net::Ipv4Addr, sync::Arc};
use cfg_if::cfg_if;
use hyper::Uri;
use serde::de::DeserializeOwned;
use tracing::{error, info, warn};

cfg_if! {
    if #[cfg(feature = "smol")] {
        use smol::lock::Mutex;

    } else if #[cfg(feature = "tokio")] {
        use tokio::sync::Mutex;

    } else {
        compile_error!("Either smol or tokio feature must be enabled");
    }
}


use crate::{dnsimple::types::{Accounts, CreateRecord, GetRecord, Records}, errors::{Error, Result}, http, Config, DnsProvider, RecordType};



const API_BASE: &str = "https:://api.dnsimple.com/v2";

pub struct Auth {
    key: String,
}

impl Auth {
    fn get_header(&self) -> String {
        format!("Bearer {}", self.key)
    }
}

struct DnSimple {
    config: Config,
    endpoint: &'static str,
    auth: Auth,
    acc_id: Mutex<Option<u32>>,
}

impl DnSimple {
    pub fn new(config: Config, auth: Auth, acc: Option<u32>) -> Self {
        Self::new_with_endpoint(config, auth, acc, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, acc: Option<u32>, endpoint: &'static str) -> Self {
        let acc_id = Mutex::new(acc);
        DnSimple {
            config,
            endpoint,
            auth,
            acc_id,
        }
    }

    async fn get_upstream_id(&self) -> Result<u32> {
        info!("Fetching account ID from upstream");
        let endpoint = format!("{}/accounts", self.endpoint);
        let uri = endpoint.parse()
            .map_err(|e| Error::UrlError(format!("Error: {endpoint} -> {e}")))?;

        let accounts_p = http::get::<Accounts>(uri, Some(self.auth.get_header())).await?;

        match accounts_p {
            Some(accounts) if accounts.accounts.len() == 1 => {
                Ok(accounts.accounts[0].id)
            }
            Some(accounts) if accounts.accounts.len() > 1 => {
                Err(Error::ApiError("More than one account returned; you must specify the account ID to use".to_string()))
            }
            // None or 0 accounts => {
            _ => {
                Err(Error::ApiError("No accounts returned from upstream".to_string()))
            }
        }
    }

    async fn get_id(&self) -> Result<u32> {
        // This is roughly equivalent to OnceLock.get_or_init(), but
        // is simpler than dealing with closure->Result and is more
        // portable.
        let mut id_p = self.acc_id.lock().await;

        if let Some(id) = *id_p {
            return Ok(id);
        }

        let id = self.get_upstream_id().await?;
        *id_p = Some(id);

        Ok(id)
    }

    async fn get_record(&self, host: &str, rtype: RecordType) -> Result<Option<GetRecord>>
    {
        let acc_id = self.get_id().await?;

        let url = format!("{}/{acc_id}/zones/{}/records?name={host}&type={rtype}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;

        let auth = self.auth.get_header();
        let recs: Records = match http::get(url, Some(auth)).await? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        // FIXME: Assumes no or single address (which probably makes sense
        // for DDNS, but may cause issues with malformed zones.
        let nr = recs.records.len();
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of IPs is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.records[0].clone()))
    }
}


impl DnsProvider for DnSimple {

    async  fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr> > {
        let rec: GetRecord = match self.get_record(host, RecordType::A).await? {
            Some(recs) => recs,
            None => return Ok(None)
        };


        Ok(Some(rec.content))
    }

    async  fn create_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()> {
        let acc_id = self.get_id().await?;

        let url = format!("{}/{acc_id}/zones/{}/records", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        let rec = CreateRecord {
            name: host.to_string(),
            rtype: RecordType::A,
            content: ip.to_string(),
            ttl: 300,
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {rec:?} to {url}");
            return Ok(())
        }
        http::post::<CreateRecord>(url, &rec, Some(auth)).await?;

        Ok(())
    }

    async  fn update_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()> {
        Ok(())
    }

    async  fn delete_v4_record(&self, host: &str) -> Result<()> {
        let rec = match self.get_record(host, RecordType::A).await? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let acc_id = self.get_id().await?;
        let rid = rec.id;

        let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        let auth = self.auth.get_header();
        http::delete(url, Some(auth)).await?;

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use random_string::charsets::ALPHANUMERIC;
    use tracing_test::traced_test;

    const TEST_API: &str = "https://api.sandbox.dnsimple.com/v2";

    fn get_client() -> DnSimple {
        let auth = Auth { key: env::var("DNSIMPLE_TOKEN").unwrap() };
        let config = Config {
            domain: env::var("DNSIMPLE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DnSimple::new_with_endpoint(config, auth, None, TEST_API)
    }

    async fn test_id_fetch() -> Result<()> {
        let client = get_client();

        let id = client.get_upstream_id().await?;
        assert_eq!(2602, id);

        Ok(())
    }

    async fn test_create_update_delete_ipv4() -> Result<()> {
        let client = get_client();

        let host = random_string::generate(16, ALPHANUMERIC);

        let ip = "1.1.1.1".parse()?;
        client.create_v4_record(&host, &ip).await?;

        let cur = client.get_v4_record(&host).await?;
        assert_eq!(Some(ip), cur);

        client.delete_v4_record(&host).await?;

        Ok(())
    }


    #[cfg(feature = "smol")]
    mod smol {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_id_fetch() -> Result<()> {
            test_id_fetch().await
        }


        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_create_update() -> Result<()> {
            test_create_update_delete_ipv4().await
        }
    }

    #[cfg(feature = "tokio")]
    mod smol {
        use super::*;

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn tokio_id_fetch() -> Result<()> {
            test_id_fetch().await
        }

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn tokio_create_update() -> Result<()> {
            test_create_update_delete_ipv4().await
        }
    }


}

