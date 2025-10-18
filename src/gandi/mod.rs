mod types;

use std::{fmt::Display};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{error, info, warn};

use types::{Record, RecordUpdate};
use crate::{errors::{Error, Result}, http, Config, DnsProvider, RecordType};

const API_BASE: &str = "https://api.gandi.net/v5/livedns";

pub enum Auth {
    ApiKey(String),
    PatKey(String),
}

impl Auth {
    fn get_header(&self) -> String {
        match self {
            Auth::ApiKey(key) => format!("Apikey {key}"),
            Auth::PatKey(key) => format!("Bearer {key}"),
        }
    }
}


pub struct Gandi {
    pub config: Config,
    pub auth: Auth,
}

impl Gandi {
    pub fn new(config: Config, auth: Auth) -> Self {
        Gandi {
            config,
            auth,
        }
    }
}

impl DnsProvider for Gandi {

    async fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();
        let mut rec: Record<T> = match http::get(url, auth).await? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        let nr = rec.rrset_values.len();

        // FIXME: Assumes no or single address (which probably makes sense
        // for DDNS, but may cause issues with malformed zones.
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of IPs is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(rec.rrset_values.remove(0)))

    }

    async fn create_record<T>(&self, rtype: RecordType, host: &str, rec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync
    {
        // PUT works for both operations
        self.update_record(rtype, host, rec).await
    }

    async fn update_record<T>(&self, rtype: RecordType, host: &str, ip: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync
    {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        let update = RecordUpdate {
            rrset_values: vec![(*ip).clone()],
            rrset_ttl: Some(300),
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PUT to {url}");
            return Ok(())
        }
        http::put::<RecordUpdate<T>>(url, &update, auth).await?;
        Ok(())
    }

    async  fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }
        http::delete(url, auth).await?;

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

    fn get_client() -> Gandi {
        let auth = if let Some(key) = env::var("GANDI_APIKEY").ok() {
            Auth::ApiKey(key)
        } else if let Some(key) = env::var("GANDI_PATKEY").ok() {
            Auth::PatKey(key)
        } else {
            panic!("No Gandi auth key set");
        };

        let config = Config {
            domain: env::var("GANDI_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };

        Gandi {
            config,
            auth,
        }
    }


    #[cfg(feature = "smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;


        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }
    }

}
