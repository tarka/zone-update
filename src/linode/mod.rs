mod types;

use std::{fmt::{Debug, Display}, sync::Mutex};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    errors::{Error, Result}, generate_helpers, http::{self, ResponseToOption, WithHeaders}, linode::types::{CreateUpdate, Domain, List, Record}, Config, DnsProvider, RecordType
};

const API_BASE: &str = "https://api.linode.com/v4/domains";

/// Authentication credentials for the Linode API.
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


/// Synchronous Linode DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct Linode {
    config: Config,
    auth: Auth,
    domain_id: Mutex<Option<u64>>,
}

impl Linode {
    /// Create a new `Linode` provider instance.
    pub fn new(config: Config, auth: Auth) -> Self {
        Self {
            config,
            auth,
            domain_id: Mutex::new(None),
        }
    }

    fn get_domain(&self) -> Result<Domain> {
        let list = http::client().get(API_BASE)
            .with_auth(self.auth.get_header())
            .with_json_headers()
            .call()?
            .to_option::<List<Domain>>()?
            .ok_or(Error::ApiError("No domains returned from upstream".to_string()))?;

        let domain = list.data.into_iter()
            .find(|d| d.domain == self.config.domain)
            .ok_or(Error::RecordNotFound(self.config.domain.clone()))?;

        Ok(domain)
    }

    fn get_domain_id(&self) -> Result<u64> {
        // This is roughly equivalent to OnceLock.get_or_init(), but
        // is simpler than dealing with closure->Result and is more
        // portable.
        let mut id_p = self.domain_id.lock()
            .map_err(|e| Error::LockingError(e.to_string()))?;

        if let Some(id) = *id_p {
            return Ok(id);
        }

        let id = self.get_domain()?.id;
        *id_p = Some(id);

        Ok(id)
    }

    fn get_upstream_records<T>(&self, rtype: &RecordType, host: &str) -> Result<Vec<Record<T>>>
        where T: DeserializeOwned
    {
        let did = self.get_domain_id()?;
        let url = format!("{API_BASE}/{did}/records");

        let mut response = http::client().get(url)
            .with_auth(self.auth.get_header())
            .with_json_headers()
            .call()?;

        // Linode returns *all* records, with no ability to filter by
        // type, resulting in a mixed-type array. To work around this
        // we filter on the raw json values before deserialising
        // properly.
        let body = response.body_mut().read_to_string()?;
        let srtype = rtype.to_string();

        let values: serde_json::Value = serde_json::from_str(&body)?;
        let data = values["data"].as_array()
            .ok_or(Error::ApiError("Data field not found".to_string()))?;
        let records = data.iter()
            .filter_map(|obj| match &obj["type"] {
                serde_json::Value::String(t)
                    if t == &srtype && obj["name"] == host
                    => Some(serde_json::from_value(obj.clone())),
                _ => None,
            })
            .collect::<std::result::Result<Vec<Record<T>>, _>>()?;

        Ok(records)
    }

    fn get_upstream_record<T>(&self, rtype: &RecordType, host: &str) -> Result<Option<Record<T>>>
        where T: DeserializeOwned
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
            warn!("No record returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.remove(0)))
    }

    fn do_delete(&self, rec: Record<String>) -> Result<()> {
        let did = self.get_domain_id()?;
        let url = format!("{API_BASE}/{did}/records/{}", rec.id);
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


impl DnsProvider for Linode {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {
         let rec = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        Ok(Some(rec.target))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let did = self.get_domain_id()?;
        let url = format!("{API_BASE}/{did}/records");

        let create = CreateUpdate {
            name: host.to_string(),
            rtype,
            target: record.to_string(),
            ttl_sec: 300,
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {create:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&create)?;
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
        let did = self.get_domain_id()?;
        let rec: Record<T> = self.get_upstream_record(&rtype, host)?
            .ok_or(Error::RecordNotFound(host.to_string()))?;
        let url = format!("{API_BASE}/{did}/records/{}", rec.id);

        let update = CreateUpdate {
            name: host.to_string(),
            rtype,
            target: urec.to_string(),
            ttl_sec: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {update:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&update)?;
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

    fn get_client() -> Linode {
        let auth = Auth {
            key: env::var("LINODE_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("LINODE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Linode::new(config, auth)
    }

    generate_tests!("test_linode");
}
