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

const API_BASE: &str = "https://api.digitalocean.com/v2/domains";

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

    fn get_upstream_records<T>(&self, rtype: &RecordType, host: &str) -> Result<Vec<Record<T>>>
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
        let recs: Records<T> = match response {
            Some(rec) => rec,
            None => return Ok(vec![])
        };

        Ok(recs.domain_records)
    }

    fn get_upstream_record<T>(&self, rtype: &RecordType, host: &str) -> Result<Option<Record<T>>>
    where
        T: DeserializeOwned
    {
        let mut recs = self.get_upstream_records(rtype, host)?;

        // FIXME: Assumes no or single address (which probably makes
        // sense for DDNS and DNS-01, but may cause issues with
        // malformed zones).
        let nr = recs.len();
        if nr > 1 {
            error!("Returned number of records is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.remove(0)))
    }

    fn do_delete(&self, rec: Record<String>) -> Result<()> {

        let url = format!("{API_BASE}/{}/records/{}", self.config.domain, rec.id);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        info!("Deleting DNS {} record {}", rec.rtype, rec.name);
        http::client().delete(url)
            .with_auth(self.auth.get_header())
            .with_json_headers()
            .call()?;

        Ok(())
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
        let rec: Record<T> = self.get_upstream_record(&rtype, host)?
            .ok_or(Error::RecordNotFound(host.to_string()))?;
        let url = format!("{API_BASE}/{}/records/{}", self.config.domain, rec.id);

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
        let rec = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("No {rtype} record to delete for {host}");
                return Ok(());
            }
        };

        self.do_delete(rec)
    }

    fn delete_all_records(&self, rtype: RecordType, host: &str) -> Result<()>
    where Self: Sized
    {
        let recs: Vec<Record<String>> = self.get_upstream_records(&rtype, host)?;
        for rec in recs {
            self.do_delete(rec)?;
        }

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
