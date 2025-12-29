pub mod errors;
mod http;

#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "dnsmadeeasy")]
pub mod dnsmadeeasy;
#[cfg(feature = "gandi")]
pub mod gandi;
#[cfg(feature = "porkbun")]
pub mod porkbun;

use std::{fmt::{self, Debug, Display, Formatter}, net::Ipv4Addr};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::warn;

use crate::errors::Result;


/// Configuration for DNS operations.
///
/// Contains the domain to operate on and a `dry_run` flag to avoid
/// making changes during testing.
pub struct Config {
    pub domain: String,
    pub dry_run: bool,
}

// This can be used by dependents of this project as part of their
// config-file, or directly. See the `netlink-ddns` project for an
// example.
/// DNS provider selection used by this crate.
///
/// Each variant contains the authentication information for the
/// selected provider.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "name")]
#[non_exhaustive]
pub enum Provider {
    Cloudflare(cloudflare::Auth),
    Gandi(gandi::Auth),
    Dnsimple(dnsimple::Auth),
    DnsMadeEasy(dnsmadeeasy::Auth),
    PorkBun(porkbun::Auth),
}

impl Provider {

    /// Return a blocking (synchronous) implementation of the selected provider.
    ///
    /// The returned boxed trait object implements `DnsProvider`.
    pub fn blocking_impl(&self, dns_conf: Config) -> Box<dyn DnsProvider> {
        match self {
            #[cfg(feature = "cloudflare")]
            Provider::Cloudflare(auth) => Box::new(cloudflare::Cloudflare::new(dns_conf, auth.clone())),
            #[cfg(feature = "gandi")]
            Provider::Gandi(auth) => Box::new(gandi::Gandi::new(dns_conf, auth.clone())),
            #[cfg(feature = "dnsimple")]
            Provider::Dnsimple(auth) => Box::new(dnsimple::Dnsimple::new(dns_conf, auth.clone(), None)),
            #[cfg(feature = "dnsmadeeasy")]
            Provider::DnsMadeEasy(auth) => Box::new(dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth.clone())),
            #[cfg(feature = "porkbun")]
            Provider::PorkBun(auth) => Box::new(porkbun::Porkbun::new(dns_conf, auth.clone())),
        }
    }

    /// Return an async implementation of the selected provider.
    ///
    /// The returned boxed trait object implements `async_impl::AsyncDnsProvider`.
    #[cfg(feature = "async")]
    pub fn async_impl(&self, dns_conf: Config) -> Box<dyn async_impl::AsyncDnsProvider> {
        match self {
            #[cfg(feature = "cloudflare")]
            Provider::Cloudflare(auth) => Box::new(async_impl::cloudflare::Cloudflare::new(dns_conf, auth.clone())),
            #[cfg(feature = "gandi")]
            Provider::Gandi(auth) => Box::new(async_impl::gandi::Gandi::new(dns_conf, auth.clone())),
            #[cfg(feature = "dnsimple")]
            Provider::Dnsimple(auth) => Box::new(async_impl::dnsimple::Dnsimple::new(dns_conf, auth.clone(), None)),
            #[cfg(feature = "dnsmadeeasy")]
            Provider::DnsMadeEasy(auth) => Box::new(async_impl::dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth.clone())),
            #[cfg(feature = "porkbun")]
            Provider::PorkBun(auth) => Box::new(async_impl::porkbun::Porkbun::new(dns_conf, auth.clone())),
        }
    }
}



#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CNAME,
    HINFO,
    MX,
    NAPTR,
    NS,
    PTR,
    SRV,
    SPF,
    SSHFP,
    TXT,
}

impl Display for RecordType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A trait for a DNS provider.
///
/// This trait defines the basic operations that a DNS provider must support.
///
/// The trait provides methods for creating, reading, updating, and
/// deleting DNS records. It also provides default implementations for
/// TXT and A records.
pub trait DnsProvider {
    /// Get a DNS record by host and record type.
    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where T: DeserializeOwned,
          Self: Sized;

    /// Create a new DNS record by host and record type.
    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where T: Serialize + DeserializeOwned + Display + Clone,
          Self: Sized;

    /// Update a DNS record by host and record type.
    fn update_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where T: Serialize + DeserializeOwned + Display + Clone,
          Self: Sized;

    /// Delete a DNS record by host and record type.
    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
    where Self: Sized;

    /// Get a TXT record.
    ///
    /// This is a helper method that calls `get_record` with the `TXT` record type.
    fn get_txt_record(&self, host: &str) -> Result<Option<String>>;

    /// Create a new TXT record.
    ///
    /// This is a helper method that calls `create_record` with the `TXT` record type.
    fn create_txt_record(&self, host: &str, record: &String) -> Result<()>;

    /// Update a TXT record.
    ///
    /// This is a helper method that calls `update_record` with the `TXT` record type.
    fn update_txt_record(&self, host: &str, record: &String) -> Result<()>;

    /// Delete a TXT record.
    ///
    /// This is a helper method that calls `delete_record` with the `TXT` record type.
    fn delete_txt_record(&self, host: &str) -> Result<()>;

    /// Get an A record.
    ///
    /// This is a helper method that calls `get_record` with the `A` record type.
    fn get_a_record(&self, host: &str) -> Result<Option<Ipv4Addr>>;

    /// Create a new A record.
    ///
    /// This is a helper method that calls `create_record` with the `A` record type.
    fn create_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()>;

    /// Update an A record.
    ///
    /// This is a helper method that calls `update_record` with the `A` record type.
    fn update_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()>;

    /// Delete an A record.
    ///
    /// This is a helper method that calls `delete_record` with the `A` record type.
    fn delete_a_record(&self, host: &str) -> Result<()>;
}

/// A macro to generate default helper implementations for provider impls.
///
/// The reason for this macro is that traits don't play well with
/// generics and Sized, preventing us from providing default
/// implementations in the trait. There are ways around this, but they
/// either involve messy downcasting or lots of match arms that need
/// to be updated as providers are added. As we want to keep the
/// process of adding providers as self-contained as possible this is
/// the simplest method for now.
#[macro_export]
macro_rules! generate_helpers {
    () => {

        fn get_txt_record(&self, host: &str) -> Result<Option<String>> {
            self.get_record::<String>(RecordType::TXT, host)
                .map(|opt| opt.map(|s| crate::strip_quotes(&s)))
        }

        fn create_txt_record(&self, host: &str, record: &String) -> Result<()> {
            self.create_record(RecordType::TXT, host, record)
        }

        fn update_txt_record(&self, host: &str, record: &String) -> Result<()> {
            self.update_record(RecordType::TXT, host, record)
        }

        fn delete_txt_record(&self, host: &str) -> Result<()> {
            self.delete_record(RecordType::TXT, host)
        }

        fn get_a_record(&self, host: &str) -> Result<Option<std::net::Ipv4Addr>> {
            self.get_record(RecordType::A, host)
        }

        fn create_a_record(&self, host: &str, record: &std::net::Ipv4Addr) -> Result<()> {
            self.create_record(RecordType::A, host, record)
        }

        fn update_a_record(&self, host: &str, record: &std::net::Ipv4Addr) -> Result<()> {
            self.update_record(RecordType::A, host, record)
        }

        fn delete_a_record(&self, host: &str) -> Result<()> {
            self.delete_record(RecordType::A, host)
        }
    }
}


fn strip_quotes(record: &str) -> String {
    let chars = record.chars();
    let mut check = chars.clone();

    let first = check.next();
    let last = check.last();

    if let Some('"') = first && let Some('"') = last {
        chars.skip(1)
            .take(record.len() - 2)
            .collect()

    } else {
        warn!("Double quotes not found in record string, using whole record.");
        record.to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use random_string::charsets::ALPHA_LOWER;
    use tracing::info;

    #[test]
    fn test_strip_quotes() -> Result<()> {
        assert_eq!("abc123".to_string(), strip_quotes("\"abc123\""));
        assert_eq!("abc123\"", strip_quotes("abc123\""));
        assert_eq!("\"abc123", strip_quotes("\"abc123"));
        assert_eq!("abc123", strip_quotes("abc123"));

        Ok(())
    }


    pub(crate) fn test_create_update_delete_ipv4(client: impl DnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        info!("Creating IPv4 {host}");
        let ip: Ipv4Addr = "1.1.1.1".parse()?;
        client.create_record(RecordType::A, &host, &ip)?;
        let cur = client.get_record(RecordType::A, &host)?;
        assert_eq!(Some(ip), cur);


        // Update
        info!("Updating IPv4 {host}");
        let ip: Ipv4Addr = "2.2.2.2".parse()?;
        client.update_record(RecordType::A, &host, &ip)?;
        let cur = client.get_record(RecordType::A, &host)?;
        assert_eq!(Some(ip), cur);


        // Delete
        info!("Deleting IPv4 {host}");
        client.delete_record(RecordType::A, &host)?;
        let del: Option<Ipv4Addr> = client.get_record(RecordType::A, &host)?;
        assert!(del.is_none());

        Ok(())
    }

    pub(crate) fn test_create_update_delete_txt(client: impl DnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        let txt = "a text reference".to_string();
        client.create_record(RecordType::TXT, &host, &txt)?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host)?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Update
        let txt = "another text reference".to_string();
        client.update_record(RecordType::TXT, &host, &txt)?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host)?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Delete
        client.delete_record(RecordType::TXT, &host)?;
        let del: Option<String> = client.get_record(RecordType::TXT, &host)?;
        assert!(del.is_none());

        Ok(())
    }

    pub(crate) fn test_create_update_delete_txt_default(client: impl DnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        let txt = "a text reference".to_string();
        client.create_txt_record(&host, &txt)?;
        let cur = client.get_txt_record(&host)?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Update
        let txt = "another text reference".to_string();
        client.update_txt_record(&host, &txt)?;
        let cur = client.get_txt_record(&host)?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Delete
        client.delete_txt_record(&host)?;
        let del = client.get_txt_record(&host)?;
        assert!(del.is_none());

        Ok(())
    }

    /// A macro to generate a standard set of tests for a DNS provider.
    ///
    /// This macro generates three tests:
    /// - `create_update_v4`: tests creating, updating, and deleting an A record.
    /// - `create_update_txt`: tests creating, updating, and deleting a TXT record.
    /// - `create_update_default`: tests creating, updating, and deleting a TXT record using the default provider methods.
    ///
    /// The tests are conditionally compiled based on the feature flag passed as an argument.
    ///
    /// # Requirements
    ///
    /// The module that uses this macro must define a `get_client()` function that returns a type
    /// that implements the `DnsProvider` trait. This function is used by the tests to get a client
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
    /// use zone_update::{generate_tests, DnsProvider};
    ///
    /// fn get_client() -> impl DnsProvider {
    ///     // ... your client implementation
    /// }
    ///
    /// // This will generate the tests, but they will only run if the "my_provider" feature is enabled.
    /// generate_tests!("my_provider");
    /// ```
    #[macro_export]
    macro_rules! generate_tests {
        ($feat:literal) => {

            #[test_log::test]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_v4() -> Result<()> {
                test_create_update_delete_ipv4(get_client())?;
                Ok(())
            }

            #[test_log::test]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_txt() -> Result<()> {
                test_create_update_delete_txt(get_client())?;
                Ok(())
            }

            #[test_log::test]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_default() -> Result<()> {
                test_create_update_delete_txt_default(get_client())?;
                Ok(())
            }
        }
    }


}
