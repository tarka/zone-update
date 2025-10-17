
mod types;

use std::fmt::Display;

use serde::de::DeserializeOwned;
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

impl Auth {
    fn get_header(&self) -> String {
        format!("Bearer {}", self.key)
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

    async fn get_domain<T>(&self) -> Result<Domain>
    where
        T: DeserializeOwned
    {

        let url = format!("{}/dns/managed/name?domainname={}", self.endpoint, self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let domain = http::get::<Domain>(url, Some(self.auth.get_header())).await?
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
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsmadeeasy"), ignore = "Dnsmadeeasy API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[traced_test]
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
