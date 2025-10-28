

pub mod errors;
mod http;

#[cfg(feature = "async")]
pub mod async_impl;

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


pub struct Config {
    pub domain: String,
    pub dry_run: bool,
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
    fn get_txt_record(&self, host: &str) -> Result<Option<String>>
    where Self: Sized
    {
        self.get_record::<String>(RecordType::TXT, host)
            .map(|opt| opt.map(|s| strip_quotes(&s)))
    }

    /// Create a new TXT record.
    ///
    /// This is a helper method that calls `create_record` with the `TXT` record type.
    fn create_txt_record(&self, host: &str, record: &String) -> Result<()>
    where Self: Sized
    {
        self.create_record(RecordType::TXT, host, record)
    }

    /// Update a TXT record.
    ///
    /// This is a helper method that calls `update_record` with the `TXT` record type.
    fn update_txt_record(&self, host: &str, record: &String) -> Result<()>
    where Self: Sized
    {
        self.update_record(RecordType::TXT, host, record)
    }

    /// Delete a TXT record.
    ///
    /// This is a helper method that calls `delete_record` with the `TXT` record type.
    fn delete_txt_record(&self, host: &str) -> Result<()>
    where Self: Sized
    {
        self.delete_record(RecordType::TXT, host)
    }

    /// Get an A record.
    ///
    /// This is a helper method that calls `get_record` with the `A` record type.
    fn get_a_record(&self, host: &str) -> Result<Option<Ipv4Addr>>
    where Self: Sized
    {
        self.get_record(RecordType::A, host)
    }

    /// Create a new A record.
    ///
    /// This is a helper method that calls `create_record` with the `A` record type.
    fn create_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()>
    where Self: Sized
    {
        self.create_record(RecordType::A, host, record)
    }

    /// Update an A record.
    ///
    /// This is a helper method that calls `update_record` with the `A` record type.
    fn update_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()>
    where Self: Sized
    {
        self.update_record(RecordType::A, host, record)
    }

    /// Delete an A record.
    ///
    /// This is a helper method that calls `delete_record` with the `A` record type.
     fn delete_a_record(&self, host: &str) -> Result<()>
    where Self: Sized
    {
        self.delete_record(RecordType::A, host)
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
