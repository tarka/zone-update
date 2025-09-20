// FIXME
#![allow(unused)]

mod types;

use std::{net::Ipv4Addr, sync::Arc};
use cfg_if::cfg_if;
use hyper::Uri;
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


use crate::{dnsimple::types::{Accounts, Records}, errors::{Error, Result}, http, Config, DnsProvider};



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
}


impl DnsProvider for DnSimple {
    async  fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr> > {
        let acc_id = self.get_id().await?;

        let url = format!("{}/{acc_id}/zones/{}/records?name={host}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;

        let auth = self.auth.get_header();
        let rec: Records = match http::get(url, Some(auth)).await? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        // FIXME: Assumes no or single address (which probably makes sense
        // for DDNS, but may cause issues with malformed zones.
        let nr = rec.records.len();
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of IPs is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(rec.records[0].content))
    }

    async  fn set_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()> {


        Ok(())
    }
}



#[cfg(test)]
#[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
mod tests {
    use super::*;
    use std::env;
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

    async fn test_get_record() -> Result<()> {
        let client = get_client();

        let ip = client.get_v4_record("test").await?;
        assert_eq!(Some("1.2.3.4".parse()?), ip);

        Ok(())
    }

    #[cfg(feature = "smol")]
    mod smol {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        async fn smol_id_fetch() -> Result<()> {
            test_id_fetch().await
        }
        #[apply(test!)]
        #[traced_test]
        async fn smol_get_record() -> Result<()> {
            test_get_record().await
        }
    }

    #[cfg(feature = "tokio")]
    mod smol {
        use super::*;

        #[tokio::test]
        #[traced_test]
        async fn tokio_id_fetch() -> Result<()> {
            test_id_fetch().await
        }
        #[tokio::test]
        #[traced_test]
        async fn tokio_get_record() -> Result<()> {
            test_get_record().await
        }
    }


}
