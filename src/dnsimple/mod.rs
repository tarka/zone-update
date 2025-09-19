// FIXME
#![allow(unused)]

mod types;

use std::{net::Ipv4Addr, sync::{LazyLock, OnceLock}};
use hyper::Uri;
use tracing::{error, info, warn};

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
    acc_id: OnceLock<u32>,
}

impl DnSimple {
    pub fn new(config: Config, auth: Auth, acc: Option<u32>) -> Self {
        Self::new_with_endpoint(config, auth, acc, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, acc: Option<u32>, endpoint: &'static str) -> Self {
        let acc_id = match acc {
            Some(id) => OnceLock::from(id),
            None => OnceLock::new(),
        };
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

    // async fn get_id(&self) -> Result<u32> {
    //     self.acc_id.get_or_init()
    // }
}



// impl DnsProvider for DnSimple {
//     async  fn get_v4_record(&self,host: &str) -> Result<Option<Ipv4Addr> > {
//     }

//     async  fn set_v4_record(&self,host: &str,ip: &Ipv4Addr) -> Result<()> {
//     }
// }



#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use macro_rules_attribute::apply;
    use smol_macros::test;
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

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
    async fn test_id_fetch() -> Result<()> {
        let client = get_client();

        let id = client.get_upstream_id().await?;
        assert_eq!(2602, id);

        Ok(())
    }


}
