mod types;

use std::{fmt::Display, str::FromStr};

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{de::DeserializeOwned, Serialize};
use sha1::{Digest, Sha1};
use tracing::{error, info, warn};
use ureq::http::header::AUTHORIZATION;

use crate::{
    dnsmadeeasy::types::Domain,
    errors::{Error, Result},
    http::{self, ResponseToOption, WithHeaders},
    Config,
    DnsProvider,
    RecordType
};


const API_BASE: &str = "https://api.dnsmadeeasy.com/V2.0";

pub struct Auth {
    key: String,
    secret: String,
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

struct DnsMadeEasy {
    config: Config,
    endpoint: &'static str,
    auth: Auth,
}

impl DnsMadeEasy {
    pub fn new(config: Config, auth: Auth) -> Self {
        Self::new_with_endpoint(config, auth, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, endpoint: &'static str) -> Self {
        Self {
            config,
            endpoint,
            auth,
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

}


impl DnsProvider for DnsMadeEasy {

    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
    where
        T: DeserializeOwned
    {
        unimplemented!()
        // let rec: GetRecord<T> = match self.get_upstream_record(rtype, host).await? {
        //     Some(recs) => recs,
        //     None => return Ok(None)
        // };


        // Ok(Some(rec.content))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        unimplemented!()
        // let acc_id = self.get_id().await?;

        // let url = format!("{}/{acc_id}/zones/{}/records", self.endpoint, self.config.domain)
        //     .parse()
        //     .map_err(|e| Erro::UrlError(format!("Error: {e}")))?;
        // let auth = self.auth.get_header();

        // let rec = CreateRecord {
        //     name: host.to_string(),
        //     rtype,
        //     content: record.to_string(),
        //     ttl: 300,
        // };

        // if self.config.dry_run {
        //     info!("DRY-RUN: Would have sent {rec:?} to {url}");
        //     return Ok(())
        // }
        // http::post::<CreateRecord>(url, &rec, Some(auth)).await?;

        // Ok(())
    }

    fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone
    {
        unimplemented!()
        // let rec: GetRecord<T> = match self.get_upstream_record(rtype, host).await? {
        //     Some(rec) => rec,
        //     None => {
        //         warn!("DELETE: Record {host} doesn't exist");
        //         return Ok(());
        //     }
        // };

        // let acc_id = self.get_id().await?;
        // let rid = rec.id;

        // let update = UpdateRecord {
        //     content: urec.to_string(),
        // };

        // let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain)
        //     .parse()
        //     .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        // if self.config.dry_run {
        //     info!("DRY-RUN: Would have sent PATCH to {url}");
        //     return Ok(())
        // }

        // let auth = self.auth.get_header();
        // http::patch(url, &update, Some(auth)).await?;

        // Ok(())
    }

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
        unimplemented!()

        // let rec: GetRecord<String> = match self.get_upstream_record(rtype, host).await? {
        //     Some(rec) => rec,
        //     None => {
        //         warn!("DELETE: Record {host} doesn't exist");
        //         return Ok(());
        //     }
        // };

        // let acc_id = self.get_id().await?;
        // let rid = rec.id;

        // let url = format!("{}/{acc_id}/zones/{}/records/{rid}", self.endpoint, self.config.domain)
        //     .parse()
        //     .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        // if self.config.dry_run {
        //     info!("DRY-RUN: Would have sent DELETE to {url}");
        //     return Ok(())
        // }

        // let auth = self.auth.get_header();
        // http::delete(url, Some(auth)).await?;

        // Ok(())
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_tests, tests::*};
    use std::env;

    const TEST_API: &str = "https://api.sandbox.dnsmadeeasy.com/V2.0";

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

    #[test]
    #[test_log::test]
    #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
    fn test_get_domain() -> Result<()> {
        let client = get_client();

        let domain = client.get_domain()?;
        assert_eq!("testcondition.net".to_string(), domain.name);

        Ok(())
    }


//    generate_tests!("test_dnsmadeeasy");
}
