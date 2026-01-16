mod types;

use std::{fmt::{Debug, Display}, sync::Mutex};

use serde::{de::DeserializeOwned, Deserialize};
use tracing::{error, info, warn};

use crate::{
    cloudflare::types::{CreateRecord, GetRecord, GetRecords, Response, ZoneInfo}, errors::{Error, Result}, generate_helpers,
    http::{self, ResponseToOption, WithHeaders}, Config, DnsProvider, RecordType
};


const API_BASE: &str = "https://api.cloudflare.com/client/v4";


/// Authentication credentials for the Cloudflare API.
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


/// Synchronous Cloudflare DNS provider implementation.
///
/// Holds configuration and authentication state for performing API calls.
pub struct Cloudflare {
    config: Config,
    auth: Auth,
    zone_id: Mutex<Option<String>>,
}

impl Cloudflare {

    /// Create a new `Cloudflare` provider instance.
    pub fn new(config: Config, auth: Auth) -> Self {
        Self {
            config,
            auth,
            zone_id: Mutex::new(None),
        }
    }

    fn get_upstream_record<T>(&self, _rtype: &RecordType, host: &str) -> Result<Option<GetRecord<T>>>
    where
        T: DeserializeOwned
    {
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/zones/{zone_id}/dns_records?name={host}.{}", self.config.domain);

        let response = http::client().get(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .to_option::<Response<GetRecords<T>>>()?;
        let mut recs = check_response(response)?;

        // FIXME: Assumes no or single address (which probably makes
        // sense for DDNS and DNS-01, but may cause issues with
        // malformed zones).
        let nr = recs.len();
        if nr > 1 {
            error!("Returned number of IPs is {nr}, should be 1");
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.remove(0)))
    }

    fn get_zone_id(&self) -> Result<String> {
        let mut id_p = self.zone_id.lock()
            .map_err(|e| Error::LockingError(e.to_string()))?;

        if let Some(id) = id_p.as_ref() {
            return Ok(id.clone());
        }

        let zone = self.get_zone_info()?;
        let id = zone.id;
        *id_p = Some(id.clone());

        Ok(id)
    }

    fn get_zone_info(&self) -> Result<ZoneInfo> {
        let uri = format!("{API_BASE}/zones?name={}", self.config.domain);
        let resp = http::client()
            .get(uri)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .to_option::<Response<Vec<ZoneInfo>>>()?;
        let mut zones = check_response(resp)?;

        Ok(zones.remove(0))
    }

}

fn check_response<T>(response: Option<Response<T>>) -> Result<T> {
    let response = match response {
        Some(r) => r,
        None => return Err(Error::RecordNotFound("Record not found".to_string())),
    };
    if !response.success {
        return Err(Error::ApiError("Failed to find record".to_string()))
    }
    Ok(response.result)
}


impl DnsProvider for Cloudflare {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {
        let resp = self.get_upstream_record(&rtype, host)?;
        let rec: GetRecord<T> = match resp {
            Some(recs) => recs,
            None => return Ok(None)
        };
        Ok(Some(rec.content))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Display,
    {
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/zones/{zone_id}/dns_records");

        let rec = CreateRecord {
            name: format!("{host}.{}", self.config.domain),
            rtype,
            content: record.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {rec:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&rec)?;
        let _response = http::client().post(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: DeserializeOwned + Display,
    {
        let rec: GetRecord<T> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("UPDATE: Record {host} doesn't exist");
                return Ok(())
            }
        };

        let rec_id = rec.id;
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/zones/{zone_id}/dns_records/{rec_id}");

        let record = CreateRecord {
            name: host.to_string(),
            rtype,
            content: urec.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PUT to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        http::client().put(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    {
        let rec: GetRecord<String> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(())
            }
        };

        let rec_id = rec.id;
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE}/zones/{zone_id}/dns_records/{rec_id}");

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

    fn get_client() -> Cloudflare {
        let auth = Auth {
            key: env::var("CLOUDFLARE_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("CLOUDFLARE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Cloudflare::new(config, auth)
    }

    generate_tests!("test_cloudflare");
}
