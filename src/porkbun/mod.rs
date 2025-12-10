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


pub(crate) const API_BASE: &str = "https://api.porkbun.com/api/json/v3/dns";

#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
    pub secret: String,
}

pub struct Porkbun {
    config: Config,
    auth: Auth,
}

impl Porkbun {
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
        let url = format!("{API_BASE}/retrieveByNameType/{}/{rtype}/{host}", self.config.domain);
        let auth = AuthOnly::from(self.auth.clone());

        let body = serde_json::to_string(&auth)?;
        let response = http::client().post(url)
            .with_json_headers()
            .send(body)?
            .to_option()?;

        // FIXME: Similar to other impls, can dedup?
        let mut recs: Records<T> = match response {
            Some(rec) => rec,
            None => return Ok(None)
        };

        // FIXME: Assumes no or single address (which probably makes
        // sense for DDNS and DNS-01, but may cause issues with
        // malformed zones).
        let nr = recs.records.len();
        if nr > 1 {
            error!("Returned number of records is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.records.remove(0)))
    }

    fn get_record_id(&self, rtype: &RecordType, host: &str) -> Result<Option<u64>> {
        let id_p = self.get_upstream_record::<String>(rtype, host)?
            .map(|r| r.id);
        Ok(id_p)
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
        let id = match self.get_record_id(&rtype, host)? {
            Some(id) => id,
            None => {
                warn!("No {rtype} record to delete for {host}");
                return Ok(());
            }
        };

        let url = format!("{API_BASE}/delete/{}/{id}", self.config.domain);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        let auth = AuthOnly::from(self.auth.clone());
        let body = serde_json::to_string(&auth)?;
        http::client().post(url)
            .with_json_headers()
            .send(body)?;

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
