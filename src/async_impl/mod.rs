use std::{fmt::Display, net::Ipv4Addr};

use serde::{de::DeserializeOwned, Serialize};

use crate::{errors::Result, RecordType};


#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "desec")]
pub mod desec;
#[cfg(feature = "digitalocean")]
pub mod digitalocean;
#[cfg(feature = "gandi")]
pub mod gandi;
#[cfg(feature = "dnsmadeeasy")]
pub mod dnsmadeeasy;
#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "porkbun")]
pub mod porkbun;


/// Asynchronous DNS provider trait.
///
/// Mirrors `DnsProvider` with async methods that can be implemented by
/// async wrappers around synchronous providers or native async clients.
#[async_trait::async_trait]
pub trait AsyncDnsProvider: Send + Sync {

    async fn get_record<T>(&self, rtype: RecordType, host: &String) -> Result<Option<T>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
        Self: Sized;

    async fn create_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static,
        Self: Sized;

    async fn update_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static,
        Self: Sized;

    async fn delete_record(&self, rtype: RecordType, host: &String) -> Result<()>
    where Self: Sized;

    async fn get_txt_record(&self, host: &String) -> Result<Option<String>>;

    async fn create_txt_record(&self, host: &String, record: &String) -> Result<()>;

    async fn update_txt_record(&self, host: &String, record: &String) -> Result<()>;

    async fn delete_txt_record(&self, host: &String) -> Result<()>;

    async fn get_a_record(&self, host: &String) -> Result<Option<Ipv4Addr>>;

    async fn create_a_record(&self, host: &String, record: &Ipv4Addr) -> Result<()>;

    async fn update_a_record(&self, host: &String, record: &Ipv4Addr) -> Result<()>;

    async fn delete_a_record(&self, host: &String) -> Result<()>;
}

#[macro_export]
macro_rules! async_provider_impl {
    ($i:ident) => {
        #[async_trait::async_trait]
        impl AsyncDnsProvider for $i {

            async fn get_record<T>(&self, rtype: RecordType, host: &String) -> Result<Option<T>>
            where
                T: DeserializeOwned + Send + Sync + 'static
            {
                let provider = self.inner.clone();
                let host = host.clone();
                unblock(move || provider.get_record(rtype, &host)).await
            }

            async fn create_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> Result<()>
            where
                T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static
            {
                let provider = self.inner.clone();
                let host = host.clone();
                let record = record.clone();
                unblock(move || provider.create_record(rtype, &host, &record)).await
            }

            async fn update_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> Result<()>
            where
                T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static
            {
                let provider = self.inner.clone();
                let host = host.clone();
                let record = record.clone();
                unblock(move || provider.update_record(rtype, &host, &record)).await
            }

            async fn delete_record(&self, rtype: RecordType, host: &String) -> Result<()>
            {
                let provider = self.inner.clone();
                let host = host.clone();
                unblock(move || provider.delete_record(rtype, &host)).await
            }

            async fn get_txt_record(&self, host: &String) -> Result<Option<String>>
            {
                self.get_record::<String>(RecordType::TXT, host).await
                    .map(|opt| opt.map(|s| crate::strip_quotes(&s)))
            }

            async fn create_txt_record(&self, host: &String, record: &String) -> Result<()>
            {
                self.create_record(RecordType::TXT, host, &crate::ensure_quotes(record)).await
            }

            async fn update_txt_record(&self, host: &String, record: &String) -> Result<()>
            {
                self.update_record(RecordType::TXT, host, &crate::ensure_quotes(record)).await
            }

            async fn delete_txt_record(&self, host: &String) -> Result<()>
            {
                self.delete_record(RecordType::TXT, host).await
            }

            async fn get_a_record(&self, host: &String) -> Result<Option<std::net::Ipv4Addr>>
            {
                self.get_record(RecordType::A, host).await
            }

            async fn create_a_record(&self, host: &String, record: &std::net::Ipv4Addr) -> Result<()>
            {
                self.create_record(RecordType::A, host, record).await
            }

            async fn update_a_record(&self, host: &String, record: &std::net::Ipv4Addr) -> Result<()>
            {
                self.update_record(RecordType::A, host, record).await
            }

            async fn delete_a_record(&self, host: &String) -> Result<()>
            {
                self.delete_record(RecordType::A, host).await
            }

        }

    };
}
pub use async_provider_impl;


#[cfg(test)]
mod tests {
    use crate::strip_quotes;

    use super::*;
    use std::net::Ipv4Addr;
    use random_string::charsets::ALPHA_LOWER;

    #[allow(unused)]
    pub async fn test_create_update_delete_ipv4(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        let ip: Ipv4Addr = "10.9.8.7".parse()?;
        client.create_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);

        // Update
        let ip: Ipv4Addr = "10.1.2.3".parse()?;
        client.update_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);

        // Delete
        client.delete_record(RecordType::A, &host).await?;
        let del: Option<Ipv4Addr> = client.get_record(RecordType::A, &host).await?;
        assert!(del.is_none());

        Ok(())
    }

    #[allow(unused)]
    pub async fn test_create_update_delete_txt(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        let txt = "\"a text reference\"".to_string();
        client.create_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, cur.unwrap());

        // Update
        let txt = "\"another text reference\"".to_string();
        client.update_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, cur.unwrap());

        // Delete
        client.delete_record(RecordType::TXT, &host).await?;
        let del: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert!(del.is_none());

        Ok(())
    }

    #[allow(unused)]
    pub async fn test_create_update_delete_txt_default(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        let txt = "a text reference".to_string();
        client.create_txt_record(&host, &txt).await?;
        let cur = client.get_txt_record(&host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));

        // Update
        let txt = "another text reference".to_string();
        client.update_txt_record(&host, &txt).await?;
        let cur = client.get_txt_record(&host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));

        // Delete
        client.delete_txt_record(&host).await?;
        let del = client.get_txt_record(&host).await?;
        assert!(del.is_none());

        Ok(())
    }


    /// A macro to generate a standard set of tests for an async DNS provider.
    ///
    /// This macro generates a suite of tests that are run against two different async runtimes: `smol` and `tokio`.
    ///
    /// For each runtime, it generates three tests:
    /// - `create_update_v4`: tests creating, updating, and deleting an A record.
    /// - `create_update_txt`: tests creating, updating, and deleting a TXT record.
    /// - `create_update_default`: tests creating, updating, and deleting a TXT record using the default provider methods.
    ///
    /// The tests are conditionally compiled based on the feature flag passed as an argument, and the
    /// `test_smol` and `test_tokio` features, which enable the tests for the respective runtimes.
    ///
    /// # Requirements
    ///
    /// The module that uses this macro must define a `get_client()` function that returns a type
    /// that implements the `AsyncDnsProvider` trait. This function is used by the tests to get a client
    /// for the DNS provider.
    ///
    /// # Arguments
    ///
    /// * `$feat` - A string literal representing the feature flag that enables these tests.
    ///
    /// # Example
    ///
    /// ```
    /// // In your test module
    /// use zone_update::async_impl::{generate_tests, AsyncDnsProvider};
    ///
    /// fn get_client() -> impl AsyncDnsProvider {
    ///     // ... your client implementation
    /// }
    ///
    /// // This will generate the tests, but they will only run if the \"my_provider\" feature is enabled,
    /// // and \"test_smol\" and/or \"test_tokio\" is enabled.
    /// generate_tests!(\"my_provider\");
    /// ```
    #[macro_export]
    macro_rules! generate_async_tests {
        ($feat:literal) => {

            #[cfg(feature = "test_smol")]
            mod smol_tests {
                use super::*;
                use crate::async_impl::tests::*;
                use macro_rules_attribute::apply;
                use smol_macros::test;

                #[apply(test!)]
                #[test_log::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_v4() -> Result<()> {
                    test_create_update_delete_ipv4(get_client()).await?;
                    Ok(())
                }

                #[apply(test!)]
                #[test_log::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_txt() -> Result<()> {
                    test_create_update_delete_txt(get_client()).await?;
                    Ok(())
                }

                #[apply(test!)]
                #[test_log::test]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_default() -> Result<()> {
                    test_create_update_delete_txt_default(get_client()).await?;
                    Ok(())
                }
            }

            #[cfg(feature = "test_tokio")]
            mod tokio_tests {
                use super::*;
                use crate::async_impl::tests::*;

                #[tokio::test]
                #[test_log::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_v4() -> Result<()> {
                    test_create_update_delete_ipv4(get_client()).await?;
                    Ok(())
                }

                #[tokio::test]
                #[test_log::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_txt() -> Result<()> {
                    test_create_update_delete_txt(get_client()).await?;
                    Ok(())
                }

                #[tokio::test]
                #[test_log::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_default() -> Result<()> {
                    test_create_update_delete_txt_default(get_client()).await?;
                    Ok(())
                }
            }

            #[cfg(feature = "test_compio")]
            mod compio_tests {
                use super::*;
                use crate::async_impl::tests::*;

                #[compio::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_v4() -> Result<()> {
                    test_create_update_delete_ipv4(get_client()).await?;
                    Ok(())
                }

                #[compio::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_txt() -> Result<()> {
                    test_create_update_delete_txt(get_client()).await?;
                    Ok(())
                }

                #[compio::test]
                #[serial_test::serial]
                #[cfg_attr(not(feature = $feat), ignore = "API test")]
                async fn create_update_default() -> Result<()> {
                    test_create_update_delete_txt_default(get_client()).await?;
                    Ok(())
                }

            }
        }
    }
}
