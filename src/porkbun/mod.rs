mod types;

use std::fmt::Display;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    errors::{Error, Result}, generate_helpers, http::{self, ResponseToOption, WithHeaders}, porkbun::types::{
        AuthOnly,
        CreateUpdate,
        Record,
        Records
    }, Config, DnsProvider, RecordType
};


const API_BASE: &str = "https://api.porkbun.com/api/json/v3/dns";

/// Authentication credentials for the Porkbun API.
///
/// Contains the API key and secret required for requests.
#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
    pub secret: String,
}

/// Synchronous Porkbun DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct Porkbun {
    config: Config,
    auth: Auth,
}

impl Porkbun {
    /// Create a new `Porkbun` provider instance.
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
        let url = format!("{API_BASE}/retrieveByNameType/{}/{rtype}/{host}", self.config.domain);
        let auth = AuthOnly::from(self.auth.clone());

        let body = serde_json::to_string(&auth)?;
        let response = http::client().post(url)
            .with_json_headers()
            .send(body)?
            .to_option()?;

        // FIXME: Similar to other impls, can dedup?
        let recs: Records<T> = match response {
            Some(rec) => rec,
            None => return Ok(vec![])
        };

        Ok(recs.records)
    }

    fn get_upstream_record<T>(&self, rtype: &RecordType, host: &str) -> Result<Option<Record<T>>>
    where
        T: DeserializeOwned
    {
        let mut recs = self.get_upstream_records(rtype, host)?;

        // FIXME (?): Assumes no or single address (which probably makes
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

    fn do_delete(&self, rec: &Record<String>) -> Result<()> {
        let url = format!("{API_BASE}/delete/{}/{}", self.config.domain, rec.id);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        info!("Deleting DNS {} record {}", rec.rtype, rec.name);
        let auth = AuthOnly::from(self.auth.clone());
        let body = serde_json::to_string(&auth)?;
        http::client().post(url)
            .with_json_headers()
            .send(body)?;

        Ok(())
    }
}


impl DnsProvider for Porkbun {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {
         let rec: Record<T> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        Ok(Some(rec.content))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let url = format!("{API_BASE}/create/{}", self.config.domain);

        let record = CreateUpdate {
            secretapikey: self.auth.secret.clone(),
            apikey: self.auth.key.clone(),
            name: host.to_string(),
            rtype,
            content: record.to_string(),
            ttl: 300,
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().post(url)
            .with_json_headers()
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let existing = match self.get_upstream_record::<T>(&rtype, host)? {
            Some(record) => record,
            None => {
                // Assume we want to create it
                return self.create_record(rtype, host, urec);
            }
        };

        let url = format!("{API_BASE}/edit/{}/{}", self.config.domain, existing.id);

        let record = CreateUpdate {
            secretapikey: self.auth.secret.clone(),
            apikey: self.auth.key.clone(),
            name: host.to_string(),
            rtype,
            content: urec.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().post(url)
            .with_json_headers()
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let recs: Vec<Record<String>> = self.get_upstream_records(&rtype, host)?;
        if recs.len() > 1 {
            error!("Returned number of records is {}, should be 1", recs.len());
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {}, should be 1", recs.len())));
         } else if recs.len() == 0 {
             warn!("No IP returned for {host}, continuing");
             return Ok(());
        }

        self.do_delete(&recs[0])?;

        Ok(())
    }

    fn delete_all_records(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let recs: Vec<Record<String>> = self.get_upstream_records(&rtype, host)?;
        for rec in recs {
            self.do_delete(&rec)?;
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

    fn get_client() -> Porkbun {
        let auth = Auth {
            key: env::var("PORKBUN_KEY").unwrap(),
            secret: env::var("PORKBUN_SECRET").unwrap(),
        };
        let config = Config {
            domain: env::var("PORKBUN_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Porkbun::new(config, auth)
    }

    generate_tests!("test_porkbun");
}
