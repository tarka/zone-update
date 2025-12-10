mod types;

use std::{fmt::Display};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{error, info, warn};

use types::{Record, RecordUpdate};
use crate::{
    errors::{Error, Result}, generate_helpers, http::{self, ResponseToOption, WithHeaders}, Config, DnsProvider, RecordType
};

pub(crate) const API_BASE: &str = "https://api.gandi.net/v5/livedns";

/// Authentication options for the Gandi provider.
///
/// Supports API key or PAT key styles depending on environment.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
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

/// Synchronous Gandi provider implementation.
///
/// Holds configuration and authentication for interacting with the Gandi API.
pub struct Gandi {
    config: Config,
    auth: Auth,
}

impl Gandi {
    /// Create a new `Gandi` provider instance.
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
            .with_json_headers()
            .with_auth(self.auth.get_header())
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
        let _response = http::client().put(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?
            .check_error()?;

        Ok(())
    }

     fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/{rtype}", self.config.domain);

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        let _response = http::client().delete(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .check_error()?;

        Ok(())
    }

    generate_helpers!();
}

#[cfg(test)]
mod tests {
    use crate::generate_tests;

    use super::*;
    use crate::tests::*;
    use std::env;

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


    generate_tests!("test_gandi");

}
