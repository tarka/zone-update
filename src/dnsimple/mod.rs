
mod types;

use std::fmt::Display;
use async_lock::Mutex;
use serde::de::DeserializeOwned;
use tracing::{error, info, warn};


use crate::{
    dnsimple::types::{
        Accounts,
        CreateRecord,
        GetRecord,
        Records,
        UpdateRecord
    },
    errors::{Error, Result},
    http,
    Config,
    DnsProvider,
    RecordType
};


const API_BASE: &str = "https://api.dnsimple.com/v2";

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

    async fn get_upstream_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<GetRecord<T>>>
    where
        T: DeserializeOwned
    {
        let acc_id = self.get_id().await?;

        let url = format!("{}/{acc_id}/zones/{}/records?name={host}&type={rtype}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;

        let auth = self.auth.get_header();
        let mut recs: Records<T> = match http::get(url, Some(auth)).await? {
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


        Ok(Some(recs.records.remove(0)))
    }
}


impl DnsProvider for DnSimple {

    async fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {
        let rec: GetRecord<T> = match self.get_upstream_record(rtype, host).await? {
            Some(recs) => recs,
            None => return Ok(None)
        };


        Ok(Some(rec.content))
    }

    async fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Display + Sync,
    {
        let acc_id = self.get_id().await?;

        let url = format!("{}/{acc_id}/zones/{}/records", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        let rec = CreateRecord {
            name: host.to_string(),
            rtype,
            content: record.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {rec:?} to {url}");
            return Ok(())
        }
        http::post::<CreateRecord>(url, &rec, Some(auth)).await?;

        Ok(())
    }

    async fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: DeserializeOwned + Display + Sync + Send,
    {
        let rec: GetRecord<T> = match self.get_upstream_record(rtype, host).await? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let acc_id = self.get_id().await?;
        let rid = rec.id;

        let update = UpdateRecord {
            content: urec.to_string(),
        };

        let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PATCH to {url}");
            return Ok(())
        }

        let auth = self.auth.get_header();
        http::patch(url, &update, Some(auth)).await?;

        Ok(())
    }

    async fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        let rec: GetRecord<String> = match self.get_upstream_record(rtype, host).await? {
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
    use crate::strip_quotes;

    use super::*;
    use crate::tests::*;
    use std::{env, net::Ipv4Addr};
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


    #[cfg(feature = "smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_id_fetch() -> Result<()> {
            test_id_fetch().await?;
            Ok(())
        }


        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn smol_create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_tests {
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
            test_create_update_delete_ipv4(get_client()).await
        }
    }


}

