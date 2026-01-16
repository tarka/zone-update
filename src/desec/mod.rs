mod types;

use std::fmt::Display;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    Config, DnsProvider, RecordType,
    desec::types::{CreateUpdateRRSet, RRSet},
    errors::{Error, Result},
    generate_helpers,
    http::{self, ResponseToOption, WithHeaders},
};

const API_BASE: &str = "https://desec.io/api/v1";

/// Authentication credentials for the deSEC API.
///
/// Contains the API key and secret required for requests.
#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
}

impl Auth {
    fn get_header(&self) -> String {
        format!("Token {}", self.key)
    }
}

/// Synchronous deSEC DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct DeSec {
    config: Config,
    auth: Auth,
}

impl DeSec {
    /// Create a new `deSEC` provider instance.
    pub fn new(config: Config, auth: Auth) -> Self {
        Self {
            config,
            auth,
        }
    }

}

impl DnsProvider for DeSec {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {

        let url = format!("{API_BASE}/domains/{}/rrsets/{host}/{rtype}/", self.config.domain);
        let response = http::client().get(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .to_option::<RRSet<T>>()?;

        let mut rec: RRSet<T> = match response {
            Some(rec) => rec,
            None => return Ok(None)
        };

        let nr = rec.records.len();

        // FIXME: Assumes no or single address (which probably makes sense
        // for DDNS, but may cause issues with malformed zones.
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of IPs is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(rec.records.remove(0)))

    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let url = format!("{API_BASE}/domains/{}/rrsets/", self.config.domain);

        let record = CreateUpdateRRSet {
            subname: host.to_string(),
            rtype,
            records: vec![record.to_string()],
            ttl: 3600, // Minimum
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().post(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let url = format!("{API_BASE}/domains/{}/rrsets/{host}/{rtype}/", self.config.domain);

        let record = CreateUpdateRRSet {
            subname: host.to_string(),
            rtype,
            records: vec![urec.to_string()],
            ttl: 3600, // Minimum
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().put(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let url = format!("{API_BASE}/domains/{}/rrsets/{host}/{rtype}/", self.config.domain);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        http::client().delete(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?;

        Ok(())
    }

    generate_helpers!();

}


#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{generate_tests, tests::*};
    use std::env;

    fn get_client() -> DeSec{
        let auth = Auth {
            key: env::var("DESEC_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("DESEC_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DeSec::new(config, auth)
    }

    generate_tests!("test_desec");
}
