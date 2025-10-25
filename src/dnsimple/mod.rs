
mod types;

use std::{fmt::Display, sync::Mutex};

use serde::de::DeserializeOwned;
use tracing::{error, info, warn};

use crate::http::{self, ResponseToOption, WithHeaders};


use crate::{
    dnsimple::types::{
        Accounts,
        CreateRecord,
        GetRecord,
        Records,
        UpdateRecord
    },
    errors::{Error, Result},
    Config,
    DnsProvider,
    RecordType
};


pub(crate) const API_BASE: &str = "https://api.dnsimple.com/v2";

pub struct Auth {
    pub key: String,
}

impl Auth {
    fn get_header(&self) -> String {
        format!("Bearer {}", self.key)
    }
}

pub struct DnSimple {
    config: Config,
    endpoint: &'static str,
    auth: Auth,
    acc_id: Mutex<Option<u32>>,
}

impl DnSimple {
    pub fn new(config: Config, auth: Auth, acc: Option<u32>) -> Self {
        Self::new_with_endpoint(config, auth, acc, API_BASE)
    }

    pub fn new_with_endpoint(config: Config, auth: Auth, acc: Option<u32>, endpoint: &'static str) -> Self {
        let acc_id = Mutex::new(acc);
        DnSimple {
            config,
            endpoint,
            auth,
            acc_id,
        }
    }

    fn get_upstream_id(&self) -> Result<u32> {
        info!("Fetching account ID from upstream");
        let url = format!("{}/accounts", self.endpoint);

        let accounts_p = http::client().get(url)
            .with_auth(self.auth.get_header())
            .call()?
            .to_option::<Accounts>()?;

        match accounts_p {
            Some(accounts) if accounts.accounts.len() == 1 => {
                Ok(accounts.accounts[0].id)
            }
            Some(accounts) if accounts.accounts.len() > 1 => {
                Err(Error::ApiError("More than one account returned; you must specify the account ID to use".to_string()))
            }
            // None or 0 accounts => {
            _ => {
                Err(Error::ApiError("No accounts returned from upstream".to_string()))
            }
        }
    }

    fn get_id(&self) -> Result<u32> {
        // This is roughly equivalent to OnceLock.get_or_init(), but
        // is simpler than dealing with closure->Result and is more
        // portable.
        let mut id_p = self.acc_id.lock()
            .map_err(|e| Error::LockingError(e.to_string()))?;

        if let Some(id) = *id_p {
            return Ok(id);
        }

        let id = self.get_upstream_id()?;
        *id_p = Some(id);

        Ok(id)
    }

    fn get_upstream_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<GetRecord<T>>>
    where
        T: DeserializeOwned
    {
        let acc_id = self.get_id()?;
        let url = format!("{}/{acc_id}/zones/{}/records?name={host}&type={rtype}", self.endpoint, self.config.domain);

        let response = http::client().get(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .call()?
            .to_option::<Records<T>>()?;
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
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        Ok(Some(recs.records.remove(0)))
    }
}


impl DnsProvider for DnSimple {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {
        let rec: GetRecord<T> = match self.get_upstream_record(rtype, host)? {
            Some(recs) => recs,
            None => return Ok(None)
        };


        Ok(Some(rec.content))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Display,
    {
        let acc_id = self.get_id()?;

        let url = format!("{}/{acc_id}/zones/{}/records", self.endpoint, self.config.domain);

        let rec = CreateRecord {
            name: host.to_string(),
            rtype,
            content: record.to_string(),
            ttl: 300,
        };

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {rec:?} to {url}");
            return Ok(())
        }

        let body = serde_json::to_string(&rec)?;
        http::client().post(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: DeserializeOwned + Display,
    {
        let rec: GetRecord<T> = match self.get_upstream_record(rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let acc_id = self.get_id()?;
        let rid = rec.id;

        let update = UpdateRecord {
            content: urec.to_string(),
        };

        let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain);
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent PATCH to {url}");
            return Ok(())
        }


        let body = serde_json::to_string(&update)?;
        http::client().patch(url)
            .with_json_headers()
            .with_auth(self.auth.get_header())
            .send(body)?;

        Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        let rec: GetRecord<String> = match self.get_upstream_record(rtype, host)? {
            Some(rec) => rec,
            None => {
                warn!("DELETE: Record {host} doesn't exist");
                return Ok(());
            }
        };

        let acc_id = self.get_id()?;
        let rid = rec.id;

        let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain);
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
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_tests, tests::*};
    use std::env;

    const TEST_API: &str = "https://api.sandbox.dnsimple.com/v2";

    fn get_client() -> DnSimple {
        let auth = Auth { key: env::var("DNSIMPLE_TOKEN").unwrap() };
        let config = Config {
            domain: env::var("DNSIMPLE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DnSimple::new_with_endpoint(config, auth, None, TEST_API)
    }

    #[test_log::test]
    #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
    fn test_id_fetch() -> Result<()> {
        let client = get_client();

        let id = client.get_upstream_id()?;
        assert_eq!(2602, id);

        Ok(())
    }

    generate_tests!("test_dnsimple");
}

