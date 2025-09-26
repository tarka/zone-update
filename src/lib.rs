mod http;
pub mod errors;


#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "gandi")]
pub mod gandi;

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

#[allow(unused)]
pub trait DnsProvider {
    fn get_record<T>(&self, rtype: RecordType, host: &str) -> impl Future<Output = Result<Option<T>>>
    where
        T: DeserializeOwned;

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    fn update_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    fn delete_record(&self, rtype: RecordType, host: &str) -> impl Future<Output = Result<()>>;


    // Default helper impls

    // We know all the types, and they're enforced above, so this lint
    // doesn't apply here(?)
    #[allow(async_fn_in_trait)]
    async fn get_txt_record(&self, host: &str) -> Result<Option<String>> {
        self.get_record::<String>(RecordType::TXT, host).await
            .map(|opt| opt.map(|s| strip_quotes(&s)))
    }

    #[allow(async_fn_in_trait)]
    async fn create_txt_record(&self, host: &str, record: &String) -> Result<()> {
        self.create_record(RecordType::TXT, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn update_txt_record(&self, host: &str, record: &String) -> Result<()> {
        self.update_record(RecordType::TXT, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn delete_txt_record(&self, host: &str) -> Result<()> {
        self.delete_record(RecordType::TXT, host).await
    }

    #[allow(async_fn_in_trait)]
    async fn get_a_record(&self, host: &str) -> Result<Option<Ipv4Addr>> {
        self.get_record(RecordType::A, host).await
    }

    #[allow(async_fn_in_trait)]
    async fn create_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()> {
        self.create_record(RecordType::A, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn update_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()> {
        self.update_record(RecordType::A, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn delete_a_record(&self, host: &str) -> Result<()> {
        self.delete_record(RecordType::A, host).await
    }
}


fn strip_quotes(record: &str) -> String {
    let chars = record.chars();
    let mut check = chars.clone();

    let first = check.nth(0);
    let last = check.last();

    if let Some('"') = first && let Some('"') = last {
        let stripped = chars.skip(1)
            .take(record.len() - 2)
            .collect();
        stripped
    } else {
        warn!("Double quotes not found in record string, using whole record.");
        record.to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_quotes() -> Result<()> {
        assert_eq!("abc123".to_string(), strip_quotes("\"abc123\""));
        assert_eq!("abc123\"", strip_quotes("abc123\""));
        assert_eq!("\"abc123", strip_quotes("\"abc123"));
        assert_eq!("abc123", strip_quotes("abc123"));

        Ok(())
    }

}
