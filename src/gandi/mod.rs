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
        let mut rec: Record<T> = match http::get(url, Some(auth)).await? {
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
        http::put::<RecordUpdate<T>>(url, &update, Some(auth)).await?;
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
        http::delete(url, Some(auth)).await?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::strip_quotes;

    use super::*;
    use std::{env, net::Ipv4Addr};
    use random_string::charsets::ALPHANUMERIC;
    use tracing_test::traced_test;

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


    // TODO: This is generic, we could move it up to top-level testing.
    async fn test_create_update_delete_ipv4() -> Result<()> {
        let client = get_client();

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let ip: Ipv4Addr = "1.1.1.1".parse()?;
        client.create_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);


        // Update
        let ip: Ipv4Addr = "2.2.2.2".parse()?;
        client.update_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);


        // Delete
        client.delete_record(RecordType::A, &host).await?;
        let del: Option<Ipv4Addr> = client.get_record(RecordType::A, &host).await?;
        assert!(del.is_none());

        Ok(())
    }

    async fn test_create_update_delete_txt() -> Result<()> {
        let client = get_client();

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let txt = "a text reference".to_string();
        client.create_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Update
        let txt = "another text reference".to_string();
        client.update_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Delete
        client.delete_record(RecordType::TXT, &host).await?;
        let del: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert!(del.is_none());

        Ok(())
    }


    #[cfg(feature = "smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
        async fn smol_create_update_a() -> Result<()> {
            test_create_update_delete_ipv4().await?;
            Ok(())
        }

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
        async fn smol_create_update_txt() -> Result<()> {
            test_create_update_delete_txt().await?;
            Ok(())
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
        async fn tokio_create_update() -> Result<()> {
            test_create_update_delete_ipv4().await?;
            test_create_update_delete_txt().await?;
            Ok(())
        }
    }

}
