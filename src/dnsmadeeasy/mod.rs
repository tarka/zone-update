
mod types;

use std::{fmt::Display, str::FromStr};

use chrono::Utc;
use ::http::{HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use sha1::{Digest, Sha1};
use tracing::{error, info, warn};

use crate::{
    http,
    dnsmadeeasy::types::Domain,
    errors::{Error, Result},
    Config,
    DnsProvider,
    RecordType,
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
    fn get_headers(&self) -> Result<Vec<(HeaderName, HeaderValue)>> {
        let time = Utc::now()
            .to_rfc2822();
        println!("TIME: {time}");
        let secret = {
            let mut hash = Sha1::new();
            hash.update(&self.secret);
            hex::encode(hash.finalize())
        };
        println!("Secret: {secret}");

        let headers = vec![
            (HeaderName::from_str(KEY_HEADER)?, HeaderValue::from_str(&self.key)?),
            (HeaderName::from_str(SECRET_HEADER)?, HeaderValue::from_str(&secret)?),
            (HeaderName::from_str(TIME_HEADER)?, HeaderValue::from_str(&time)?),
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

    async fn get_domain(&self) -> Result<Domain>
    {
        let url = format!("{}/dns/managed/name?domainname={}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        println!("URL: {url}");
        let domain = http::get_with_headers::<Domain>(url, self.auth.get_headers()?).await?
            .ok_or(Error::ApiError("No accounts returned from upstream".to_string()))?;

        Ok(domain)
    }

    // async fn get_upstream_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<GetRecord<T>>>
    // where
    //     T: DeserializeOwned
    // {

    //     let url = format!("{}/{acc_id}/zones/{}/records?name={host}&type={rtype}", self.endpoint, self.config.domain)
    //         .parse()
    //         .map_err(|e| Error::UrlError(format!("Error: {e}")))?;

    // }

}


impl DnsProvider for DnsMadeEasy {

    async fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T> >
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

    async fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Display + Sync,
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

    async fn update_record<T>(&self, rtype: RecordType, host: &str, urec: &T) -> Result<()>
    where
        T: DeserializeOwned + Display + Sync + Send,
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

    async fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()> {
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
    use crate::tests::*;
    use std::env;
    use tracing_test::traced_test;

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


    #[cfg(feature = "smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn test_get_domain() -> Result<()> {
            let client = get_client();

            let domain = client.get_domain().await?;
            assert_eq!("testcondition.net".to_string(), domain.name);

            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }

    }

}
