mod types;

use std::fmt::Display;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    Config, DnsProvider, RecordType,
    digitalocean::types::{CreateUpdate, Record, Records},
    errors::{Error, Result},
    generate_helpers,
    http::{self, ResponseToOption, WithHeaders},
};

const API_BASE: &'static str = "https://api.digitalocean.com/v2/domains";

/// Authentication credentials for the Digital Ocean API.
///
/// Contains the API key and secret required for requests.
#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
}

impl Auth {
    fn get_header(&self) -> String {
        format!("Bearer {}", self.key)
    }
}

/// Synchronous DigitalOcean DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct DigitalOcean {
    config: Config,
    auth: Auth,
}

impl DigitalOcean {
    /// Create a new `Digital Ocean` provider instance.
    pub fn new(config: Config, auth: Auth) -> Self {
        Self {
            config,
            auth,
        }
    }

    fn get_upstream_record<T>(&self, rtype: &RecordType, host: &str) -> Result<Option<Record<T>>>
    where
        T: DeserializeOwned
    {
        let url = format!("{API_BASE}/{}/records?type={rtype}&name={host}.{}", self.config.domain, self.config.domain);

        let response = http::client().get(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .to_option()?;

        // FIXME: Similar to other impls, can dedup?
        let mut recs: Records<T> = match response {
            Some(rec) => rec,
            None => return Ok(None)
        };

        // FIXME: Assumes no or single address (which probably makes
        // sense for DDNS and DNS-01, but may cause issues with
        // malformed zones).
        let nr = recs.domain_records.len();
        if nr > 1 {
            error!("Returned number of records is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.domain_records.remove(0)))
    }

    fn get_record_id(&self, rtype: &RecordType, host: &str) -> Result<Option<u64>> {
        let id_p = self.get_upstream_record::<String>(rtype, host)?
            .map(|r| r.id);
        Ok(id_p)
    }
}

impl DnsProvider for DigitalOcean {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {
         let rec: Record<T> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        Ok(Some(rec.data))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let url = format!("{API_BASE}/{}/records", self.config.domain);

        let record = CreateUpdate {
            name: host.to_string(),
            rtype,
            data: record.to_string(),
            ttl: 300,
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().post(url)
            .with_auth(self.auth.get_header())
            .with_json_headers()
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let id = self.get_record_id(&rtype, host)?
            .ok_or(Error::RecordNotFound(host.to_string()))?;
        let url = format!("{API_BASE}/{}/records/{id}", self.config.domain);

        let record = CreateUpdate {
            name: host.to_string(),
            rtype,
            data: urec.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().put(url)
            .with_auth(self.auth.get_header())
            .with_json_headers()
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let id = match self.get_record_id(&rtype, host)? {
            Some(id) => id,
            None => {
                warn!("No {rtype} record to delete for {host}");
                return Ok(());
            }
        };

        let url = format!("{API_BASE}/{}/records/{id}", self.config.domain);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        http::client().delete(url)
            .with_auth(self.auth.get_header())
            .with_json_headers()
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

    fn get_client() -> DigitalOcean {
        let auth = Auth {
            key: env::var("DIGITALOCEAN_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("DIGITALOCEAN_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DigitalOcean::new(config, auth)
    }

    generate_tests!("test_digitalocean");
}
