use std::sync::Arc;
use std::{fmt::Display, net::Ipv4Addr};

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::dnsimple::{Auth, DnSimple, API_BASE};
use crate::{Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;

struct AsyncDnSimple {
    inner: Arc<DnSimple>,
}

impl AsyncDnSimple {
    pub fn new(config: Config, auth: Auth, acc: Option<u32>) -> Self {
        Self::new_with_endpoint(config, auth, acc, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, acc: Option<u32>, endpoint: &'static str) -> Self {
        let inner = DnSimple::new_with_endpoint(config, auth, acc, endpoint);
        Self {
            inner: Arc::new(inner)
        }
    }
}

impl AsyncDnsProvider for AsyncDnSimple {

    fn get_record<T>(&self, rtype: RecordType, host: &String) -> impl Future<Output = Result<Option<T>>>
    where
        T: DeserializeOwned + Send + Sync + 'static
    {
        let provider = self.inner.clone();
        let host = host.clone();
        unblock(move || provider.get_record(rtype, &host))
    }

    fn create_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static
{
        let provider = self.inner.clone();
        let host = host.clone();
        let record = record.clone();
        unblock(move || provider.create_record(rtype, &host, &record))
    }

    fn update_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static
    {
        let provider = self.inner.clone();
        let host = host.clone();
        let record = record.clone();
        unblock(move || provider.update_record(rtype, &host, &record))
    }

    fn delete_record(&self, rtype: RecordType, host: &String) -> impl Future<Output = Result<()>>
    {
        let provider = self.inner.clone();
        let host = host.clone();
        unblock(move || provider.delete_record(rtype, &host))
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_impl::tests::*;
    use std::env;

    const TEST_API: &str = "https://api.sandbox.dnsimple.com/v2";

    fn get_client() -> AsyncDnSimple {
        let auth = Auth { key: env::var("DNSIMPLE_TOKEN").unwrap() };
        let config = Config {
            domain: env::var("DNSIMPLE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        AsyncDnSimple::new_with_endpoint(config, auth, None, TEST_API)
    }


    #[cfg(feature = "test_smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;


        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[apply(test!)]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }
    }

    #[cfg(feature = "test_tokio")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_v4() -> Result<()> {
            test_create_update_delete_ipv4(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_txt() -> Result<()> {
            test_create_update_delete_txt(get_client()).await?;
            Ok(())
        }

        #[tokio::test]
        #[test_log::test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn create_update_default() -> Result<()> {
            test_create_update_delete_txt_default(get_client()).await?;
            Ok(())
        }

    }


}

