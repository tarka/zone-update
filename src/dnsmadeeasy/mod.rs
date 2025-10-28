mod types;

use std::{fmt::Display, sync::Mutex};

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha1::Sha1;
use tracing::{error, info, warn};

use crate::{
    dnsmadeeasy::types::{Domain, Record, Records}, errors::{Error, Result}, generate_helpers, http::{self, ResponseToOption, WithHeaders}, Config, DnsProvider, RecordType
};


pub(crate) const API_BASE: &str = "https://api.dnsmadeeasy.com/V2.0";

#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
    pub key: String,
    pub secret: String,
}

// See https://api-docs.dnsmadeeasy.com/
const KEY_HEADER: &str = "x-dnsme-apiKey";
const SECRET_HEADER: &str = "x-dnsme-hmac";
const TIME_HEADER: &str = "x-dnsme-requestDate";


impl Auth {
    fn get_headers(&self) -> Result<Vec<(&str, String)>> {
        // See https://api-docs.dnsmadeeasy.com/
        let time = Utc::now()
            .to_rfc2822();
        let hmac = {
            let secret = self.secret.clone().into_bytes();
            let mut mac = Hmac::<Sha1>::new_from_slice(&secret)
                .map_err(|e| Error::AuthError(format!("Error generating HMAC: {e}")))?;
            mac.update(&time.clone().into_bytes());
            hex::encode(mac.finalize().into_bytes())
        };
        let headers = vec![
            (KEY_HEADER, self.key.clone()),
            (SECRET_HEADER, hmac),
            (TIME_HEADER, time),
        ];

        Ok(headers)
    }
}

pub struct DnsMadeEasy {
    config: Config,
    endpoint: &'static str,
    auth: Auth,
    domain_id: Mutex<Option<u32>>,
}

impl DnsMadeEasy {
    pub fn new(config: Config, auth: Auth) -> Self {
        Self::new_with_endpoint(config, auth, API_BASE)
    }

    pub fn new_with_endpoint(config: Config, auth: Auth, endpoint: &'static str) -> Self {
        Self {
            config,
            endpoint,
            auth,
            domain_id: Mutex::new(None),
        }
    }

    fn get_domain(&self) -> Result<Domain>
    {
        let url = format!("{}/dns/managed/name?domainname={}", self.endpoint, self.config.domain);

        let domain = http::client().get(url)
            .with_headers(self.auth.get_headers()?)?
            .call()?
            .to_option::<Domain>()?
            .ok_or(Error::ApiError("No domain returned from upstream".to_string()))?;

        Ok(domain)
    }

    fn get_domain_id(&self) -> Result<u32> {
        // This is roughly equivalent to OnceLock.get_or_init(), but
        // is simpler than dealing with closure->Result and is more
        // portable.
        let mut id_p = self.domain_id.lock()
            .map_err(|e| Error::LockingError(e.to_string()))?;

        if let Some(id) = *id_p {
            return Ok(id);
        }

        let domain = self.get_domain()?;
        let id = domain.id;
        *id_p = Some(id);

        Ok(id)
    }


    fn get_upstream_record<T>(&self, rtype: &RecordType, host: &str) -> Result<Option<Record<T>>>
    where
        T: DeserializeOwned
    {
        let domain_id = self.get_domain_id()?;
        let url = format!("{}/dns/managed/{domain_id}/records?recordName={host}&type={rtype}", self.endpoint);

        let response = http::client().get(url)
            .with_json_headers()
            .with_headers(self.auth.get_headers()?)?
            .call()?
            .to_option::<Records<T>>()?;

        // FIXME: Similar to the dnsimple impl, can dedup?
        let mut recs: Records<T> = match response {
            Some(rec) => rec,
            None => return Ok(None)
        };

        // FIXME: Assumes no or single address (which probably makes
        // sense for DDNS and DNS-01, but may cause issues with
        // malformed zones).
        let nr = recs.records.len();
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of records is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No record returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.records.remove(0)))
    }
}


impl DnsProvider for DnsMadeEasy {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {

        let rec: Record<T> = match self.get_upstream_record(&rtype, host)? {
            Some(recs) => recs,
            None => return Ok(None)
        };

        Ok(Some(rec.value))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let domain_id = self.get_domain_id()?;
        let url = format!("{}/dns/managed/{domain_id}/records", self.endpoint);

        let record = Record {
            id: 0,
            name: host.to_string(),
            value: record.to_string(),
            rtype,
            source_id: 0,
            ttl: 300,
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().post(url)
            .with_json_headers()
            .with_headers(self.auth.get_headers()?)?
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        let rec: Record<String> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let rid = rec.id;
        let domain_id = self.get_domain_id()?;
        let url = format!("{}/dns/managed/{domain_id}/records/{rid}", self.endpoint);

        let record = Record {
            id: 0,
            name: host.to_string(),
            value: urec.to_string(),
            rtype,
            source_id: 0,
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {record:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&record)?;
        let _response = http::client().put(url)
            .with_json_headers()
            .with_headers(self.auth.get_headers()?)?
            .send(body)?
            .check_error()?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {

        let rec: Record<String> = match self.get_upstream_record(&rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let rid = rec.id;
        let domain_id = self.get_domain_id()?;
        let url = format!("{}/dns/managed/{domain_id}/records/{rid}", self.endpoint);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }

        let _response = http::client().delete(url)
            .with_json_headers()
            .with_headers(self.auth.get_headers()?)?
            .call()?
            .check_error()?;

        Ok(())
    }


    generate_helpers!();
}




#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{generate_tests, tests::*};
    use std::env;

    pub(crate) const TEST_API: &str = "https://api.sandbox.dnsmadeeasy.com/V2.0";

    fn get_client() -> DnsMadeEasy {
        let auth = Auth {
            key: env::var("DNSMADEEASY_KEY").unwrap(),
            secret: env::var("DNSMADEEASY_SECRET").unwrap(),
        };
        let config = Config {
            domain: env::var("DNSMADEEASY_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DnsMadeEasy::new_with_endpoint(config, auth, TEST_API)
    }

    #[test_log::test]
    #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
    fn test_get_domain() -> Result<()> {
        let client = get_client();

        let domain = client.get_domain()?;
        assert_eq!("testcondition.net".to_string(), domain.name);

        Ok(())
    }


    generate_tests!("test_dnsmadeeasy");
}
