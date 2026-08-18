mod types;

use std::{fmt::{Debug, Display}, sync::Mutex};

use serde::{de::DeserializeOwned, Deserialize};
use tracing::{error, info, warn};

use crate::{
    Config, DnsProvider, RecordType,
    bunny::types::{CreateUpdate, Record, ZoneInfo, ZoneList},
    errors::{Error, Result},
    generate_helpers,
    http::{self, ResponseToOption, WithHeaders},
};

const API_BASE: &str = "https://api.bunny.net/dnszone";


/// Authentication credentials for the Bunny API.
///
/// Contains the API key and secret required for requests.
#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
}

impl Auth {
    fn get_header(&self) -> String {
         self.key.clone()
    }
}


/// Synchronous Bunny DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct Bunny {
    config: Config,
    auth: Auth,
    zone_id: Mutex<Option<u64>>,
}

impl Bunny {

    /// Create a new `Bunny` provider instance.
    pub fn new(config: Config, auth: Auth) -> Self {
        Self {
            config,
            auth,
            zone_id: Mutex::new(None),
        }
    }


    fn get_zone_id(&self) -> Result<u64> {
        let mut id_p = self.zone_id.lock()
            .map_err(|e| Error::LockingError(e.to_string()))?;

        if let Some(id) = id_p.as_ref() {
            return Ok(*id);
        }

        let zone = self.get_zone_info()?;
        let id = zone.id;
        *id_p = Some(id);

        Ok(id)
    }

    fn get_zone_info(&self) -> Result<ZoneInfo> {
        let uri = format!("{API_BASE}?search={}", self.config.domain);
        let zones = http::client()
            .get(uri)
            .with_json_headers()
            .header("AccessKey", self.auth.get_header())
            .call()?
            .to_option::<ZoneList>()?
            .ok_or(Error::RecordNotFound(format!("Couldn't fetch zone info for {}", self.config.domain)))?
            .items;
        let zone = zones.into_iter()
            .find(|z| z.domain == self.config.domain)
            .ok_or(Error::RecordNotFound(format!("Couldn't fetch zone info for {}", self.config.domain)))?;

        Ok(zone)
    }

    fn get_upstream_records<T>(&self, rtype: RecordType, host: &str) -> Result<Vec<Record<T>>>
    where
        T: DeserializeOwned
    {
        println!("GET UPSTREAM {rtype}, {host}");
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/{zone_id}");

        let mut response = http::client().get(url)
            .header("AccessKey", self.auth.get_header())
            .with_json_headers()
            .call()?;

        // Bunny returns *all* records, with no ability to filter by
        // type, resulting in a mixed-type array. To work around this
        // we filter on the raw json values before deserialising
        // properly.
        let body = response.body_mut().read_to_string()?;
        let u64rtype = u64::from(rtype);

        let values: serde_json::Value = serde_json::from_str(&body)?;
        let data = values["Records"].as_array()
            .ok_or(Error::ApiError("Data field not found".to_string()))?;
        let records = data.iter()
            .filter_map(|obj| match &obj["Type"] {
                serde_json::Value::Number(n)
                    if n.as_u64().is_some_and(|v| v == u64rtype) && obj["Name"] == host
                    => Some(serde_json::from_value(obj.clone())),
                _ => None,
            })
            .collect::<std::result::Result<Vec<Record<T>>, _>>()?;
        println!("DONE");

        Ok(records)
    }

    fn get_upstream_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<Record<T>>>
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
            warn!("No record returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.remove(0)))
    }

    fn do_delete(&self, rec: Record<String>) -> Result<()> {
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/{zone_id}/records/{}", rec.id);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        info!("Deleting DNS {} record {}", rec.rtype, rec.name);
        http::client().delete(url)
            .with_json_headers()
            .header("AccessKey", self.auth.get_header())
            .call()?;

        Ok(())
    }

}


impl DnsProvider for Bunny {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {
        let resp = self.get_upstream_record(rtype, host)?;
        let rec: Record<T> = match resp {
            Some(recs) => recs,
            None => return Ok(None)
        };
        Ok(Some(rec.value))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Display,
    {
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/{zone_id}/records");

        let rec = CreateUpdate {
            name: host.to_string(),
            rtype,
            value: record.to_string(),
            ttl: 300,
        };

        let body = serde_json::to_string(&rec)?;

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {body} to {url}");
            return Ok(())
        }

        let _response = http::client().put(url)
            .with_json_headers()
            .header("AccessKey", self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: DeserializeOwned + Display,
    {
        let rec: Record<T> = match self.get_upstream_record(rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("UPDATE: Record {host} doesn't exist");
                return Ok(())
            }
        };

        let rec_id = rec.id;
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/{zone_id}/records/{rec_id}");

        let record = CreateUpdate {
            name: host.to_string(),
            rtype,
            value: urec.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PUT to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        http::client().post(url)
            .with_json_headers()
            .header("AccessKey", self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let rec = match self.get_upstream_record(rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(())
            }
        };

        self.do_delete(rec)
    }

    fn delete_all_records(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let recs: Vec<Record<String>> = self.get_upstream_records(rtype, host)?;
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

    fn get_client() -> Bunny {
        let auth = Auth {
            key: env::var("BUNNY_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("BUNNY_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Bunny::new(config, auth)
    }

    generate_tests!("test_bunny");
}
