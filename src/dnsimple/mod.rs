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


use crate::{dnsimple::types::Accounts, errors::{Error, Result}, http, Config, DnsProvider};



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



// impl DnsProvider for DnSimple {
//     async  fn get_v4_record(&self,host: &str) -> Result<Option<Ipv4Addr> > {
//     }

//     async  fn set_v4_record(&self,host: &str,ip: &Ipv4Addr) -> Result<()> {
//     }
// }



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
    }

    #[cfg(feature = "tokio")]
    mod smol {
        use super::*;

        #[tokio::test]
        #[traced_test]
        async fn smol_id_fetch() -> Result<()> {
            test_id_fetch().await
        }
    }


}
