#![doc = include_str!("../README.md")]

pub mod errors;
mod http;

#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "bunny")]
pub mod bunny;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "desec")]
pub mod desec;
#[cfg(feature = "digitalocean")]
pub mod digitalocean;
#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "dnsmadeeasy")]
pub mod dnsmadeeasy;
#[cfg(feature = "gandi")]
pub mod gandi;
#[cfg(feature = "linode")]
pub mod linode;
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

/// DNS provider selection used by this crate.
///
/// Each variant contains the authentication information for the
/// selected provider.
///
/// This can be used by dependents of this project as part of their
/// config-file, or directly. See the `netlink-ddns` project for an
/// example.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "name")]
#[non_exhaustive]
pub enum Provider {
    Bunny(bunny::Auth),
    Cloudflare(cloudflare::Auth),
    DeSec(desec::Auth),
    DigitalOcean(digitalocean::Auth),
    DnsMadeEasy(dnsmadeeasy::Auth),
    Dnsimple(dnsimple::Auth),
    Gandi(gandi::Auth),
    Linode(linode::Auth),
    PorkBun(porkbun::Auth),
}

impl Provider {

    /// Return a blocking (synchronous) implementation of the selected provider.
    ///
    /// The returned boxed trait object implements `DnsProvider`.
    pub fn blocking_impl(&self, dns_conf: Config) -> Box<dyn DnsProvider> {
        match self {
            #[cfg(feature = "bunny")]
            Provider::Bunny(auth) => Box::new(bunny::Bunny::new(dns_conf, auth.clone())),
            #[cfg(feature = "cloudflare")]
            Provider::Cloudflare(auth) => Box::new(cloudflare::Cloudflare::new(dns_conf, auth.clone())),
            #[cfg(feature = "desec")]
            Provider::DeSec(auth) => Box::new(desec::DeSec::new(dns_conf, auth.clone())),
            #[cfg(feature = "digitalocean")]
            Provider::DigitalOcean(auth) => Box::new(digitalocean::DigitalOcean::new(dns_conf, auth.clone())),
            #[cfg(feature = "gandi")]
            Provider::Gandi(auth) => Box::new(gandi::Gandi::new(dns_conf, auth.clone())),
            #[cfg(feature = "dnsimple")]
            Provider::Dnsimple(auth) => Box::new(dnsimple::Dnsimple::new(dns_conf, auth.clone(), None)),
            #[cfg(feature = "dnsmadeeasy")]
            Provider::DnsMadeEasy(auth) => Box::new(dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth.clone())),
            #[cfg(feature = "porkbun")]
            Provider::PorkBun(auth) => Box::new(porkbun::Porkbun::new(dns_conf, auth.clone())),
            #[cfg(feature = "linode")]
            Provider::Linode(auth) => Box::new(linode::Linode::new(dns_conf, auth.clone())),
        }
    }

    /// Return an async implementation of the selected provider.
    ///
    /// The returned boxed trait object implements `async_impl::AsyncDnsProvider`.
    #[cfg(feature = "async")]
    pub fn async_impl(&self, dns_conf: Config) -> Box<dyn async_impl::AsyncDnsProvider> {
        match self {
            #[cfg(feature = "bunny")]
            Provider::Bunny(auth) => Box::new(async_impl::bunny::Bunny::new(dns_conf, auth.clone())),
            #[cfg(feature = "cloudflare")]
            Provider::Cloudflare(auth) => Box::new(async_impl::cloudflare::Cloudflare::new(dns_conf, auth.clone())),
            #[cfg(feature = "desec")]
            Provider::DeSec(auth) => Box::new(async_impl::desec::DeSec::new(dns_conf, auth.clone())),
            #[cfg(feature = "digitalocean")]
            Provider::DigitalOcean(auth) => Box::new(async_impl::digitalocean::DigitalOcean::new(dns_conf, auth.clone())),
            #[cfg(feature = "gandi")]
            Provider::Gandi(auth) => Box::new(async_impl::gandi::Gandi::new(dns_conf, auth.clone())),
            #[cfg(feature = "dnsimple")]
            Provider::Dnsimple(auth) => Box::new(async_impl::dnsimple::Dnsimple::new(dns_conf, auth.clone(), None)),
            #[cfg(feature = "dnsmadeeasy")]
            Provider::DnsMadeEasy(auth) => Box::new(async_impl::dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth.clone())),
            #[cfg(feature = "porkbun")]
            Provider::PorkBun(auth) => Box::new(async_impl::porkbun::Porkbun::new(dns_conf, auth.clone())),
            #[cfg(feature = "linode")]
            Provider::Linode(auth) => Box::new(async_impl::linode::Linode::new(dns_conf, auth.clone())),
        }
    }
}



#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CNAME,
    MX,
    NS,
    PTR,
    SRV,
    TXT,
    SVCB,
    HTTPS,
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
    fn create_txt_record(&self, host: &str, record: &str) -> Result<()>;

    /// Update a TXT record.
    ///
    /// This is a helper method that calls `update_record` with the `TXT` record type.
    fn update_txt_record(&self, host: &str, record: &str) -> Result<()>;

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
                .map(|opt| opt.map(|s| $crate::strip_quotes(&s)))
        }

        fn create_txt_record(&self, host: &str, record: &str) -> Result<()> {
            self.create_record(RecordType::TXT, host, &$crate::ensure_quotes(record))
        }

        fn update_txt_record(&self, host: &str, record: &str) -> Result<()> {
            self.update_record(RecordType::TXT, host, &$crate::ensure_quotes(record))
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

fn ensure_quotes(record: &str) -> String {
    let starts = record.starts_with('"');
    let ends = record.ends_with('"');

    match (starts, ends) {
        (true, true)   => record.to_string(),
        (true, false)  => format!("{}\"", record),
        (false, true)  => format!("\"{}", record),
        (false, false) => format!("\"{}\"", record),
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
    fn test_strip_quotes() {
        assert_eq!("abc123".to_string(), strip_quotes("\"abc123\""));
        assert_eq!("abc123\"", strip_quotes("abc123\""));
        assert_eq!("\"abc123", strip_quotes("\"abc123"));
        assert_eq!("abc123", strip_quotes("abc123"));
    }

    #[test]
    fn test_already_quoted() {
        assert_eq!(ensure_quotes(&"\"hello\"".to_string()), "\"hello\"");
        assert_eq!(ensure_quotes(&"\"\"".to_string()), "\"\"");
        assert_eq!(ensure_quotes(&"\"a\"".to_string()), "\"a\"");
        assert_eq!(ensure_quotes(&"\"quoted \" string\"".to_string()), "\"quoted \" string\"");
    }

    #[test]
    fn test_no_quotes() {
        assert_eq!(ensure_quotes(&"hello".to_string()), "\"hello\"");
        assert_eq!(ensure_quotes(&"".to_string()), "\"\"");
        assert_eq!(ensure_quotes(&"a".to_string()), "\"a\"");
        assert_eq!(ensure_quotes(&"hello world".to_string()), "\"hello world\"");
    }

    #[test]
    fn test_only_starting_quote() {
        assert_eq!(ensure_quotes(&"\"hello".to_string()), "\"hello\"");
        assert_eq!(ensure_quotes(&"\"test case".to_string()), "\"test case\"");
    }

    #[test]
    fn test_only_ending_quote() {
        assert_eq!(ensure_quotes(&"hello\"".to_string()), "\"hello\"");
        assert_eq!(ensure_quotes(&"test case\"".to_string()), "\"test case\"");
    }

    #[test]
    fn test_whitespace_handling() {
        // Empty and whitespace-only strings become empty quoted strings
        assert_eq!(ensure_quotes(&"".to_string()), "\"\"");
        assert_eq!(ensure_quotes(&"   ".to_string()), "\"   \"");
        assert_eq!(ensure_quotes(&"\t\n".to_string()), "\"\t\n\"");
        // Whitespace within content is preserved
        assert_eq!(ensure_quotes(&" hello ".to_string()), "\" hello \"");
        assert_eq!(ensure_quotes(&"\" hello ".to_string()), "\" hello \"");
        assert_eq!(ensure_quotes(&" hello \"".to_string()), "\" hello \"");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(ensure_quotes(&"hello\nworld".to_string()), "\"hello\nworld\"");
        assert_eq!(ensure_quotes(&"hello\tworld".to_string()), "\"hello\tworld\"");
        assert_eq!(ensure_quotes(&"123!@#$%^&*()".to_string()), "\"123!@#$%^&*()\"");
    }

    pub(crate) fn test_create_update_delete_ipv4(client: impl DnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHA_LOWER);

        // Create
        info!("Creating IPv4 {host}");
        let ip: Ipv4Addr = "10.9.8.7".parse()?;
        client.create_record(RecordType::A, &host, &ip)?;
        info!("Checking IPv4 {host}");
        let cur = client.get_record(RecordType::A, &host)?;
        assert_eq!(Some(ip), cur);


        // Update
        info!("Updating IPv4 {host}");
        let ip: Ipv4Addr = "10.10.9.8".parse()?;
        client.update_record(RecordType::A, &host, &ip)?;
        info!("Checking IPv4 {host}");
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
        let txt = "\"a text reference\"".to_string();
        client.create_record(RecordType::TXT, &host, &txt)?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host)?;
        assert_eq!(txt, cur.unwrap());


        // Update
        let txt = "\"another text reference\"".to_string();
        client.update_record(RecordType::TXT, &host, &txt)?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host)?;
        assert_eq!(txt, cur.unwrap());


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
            use serial_test::serial;

            #[test_log::test]
            #[serial]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_v4() -> Result<()> {
                test_create_update_delete_ipv4(get_client())?;
                Ok(())
            }

            #[test_log::test]
            #[serial]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_txt() -> Result<()> {
                test_create_update_delete_txt(get_client())?;
                Ok(())
            }

            #[test_log::test]
            #[serial]
            #[cfg_attr(not(feature = $feat), ignore = "API test")]
            fn create_update_default() -> Result<()> {
                test_create_update_delete_txt_default(get_client())?;
                Ok(())
            }
        }
    }


}
