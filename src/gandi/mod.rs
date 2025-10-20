mod types;

use std::{fmt::Display};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{error, info, warn};

use types::{Record, RecordUpdate};
use ureq::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use crate::{errors::{Error, Result}, http::{self, ResponseToOption}, Config, DnsProvider, RecordType};

pub(crate) const API_BASE: &str = "https://api.gandi.net/v5/livedns";

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
    config: Config,
    auth: Auth,
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

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {

        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain);
        let response = http::client().get(url)
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, self.auth.get_header())
            .call()?
            .to_option::<Record<T>>()?;

        let mut rec: Record<T> = match response {
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

    fn create_record<T>(&self, rtype: RecordType, host: &str, rec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        // PUT works for both operations
        self.update_record(rtype, host, rec)
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, ip: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PUT to {url}");
            return Ok(())
        }

        let update = RecordUpdate {
            rrset_values: vec![(*ip).clone()],
            rrset_ttl: Some(300),
        };

        let body = serde_json::to_string(&update)?;
        http::client().put(url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, self.auth.get_header())
            .send(body)?;

        Ok(())
    }

     fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain);

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        http::client().delete(url)
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, self.auth.get_header())
            .call()?;

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


    #[test]
    #[test_log::test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    fn create_update_v4() -> Result<()> {
        test_create_update_delete_ipv4(get_client())?;
        Ok(())
    }

    #[test]
    #[test_log::test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    fn create_update_txt() -> Result<()> {
        test_create_update_delete_txt(get_client())?;
        Ok(())
    }

    #[test]
    #[test_log::test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    fn create_update_default() -> Result<()> {
        test_create_update_delete_txt_default(get_client())?;
        Ok(())
    }

}
