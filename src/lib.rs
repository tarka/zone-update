
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
    fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned;

    fn create_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    fn update_record<T>(&self, rtype: RecordType, host: &str, record: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>;


    // Default helper impls

    // We know all the types, and they're enforced above, so this lint
    // doesn't apply here(?)
    fn get_txt_record(&self, host: &str) -> Result<Option<String>> {
        self.get_record::<String>(RecordType::TXT, host)
            .map(|opt| opt.map(|s| strip_quotes(&s)))
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

    fn get_a_record(&self, host: &str) -> Result<Option<Ipv4Addr>> {
        self.get_record(RecordType::A, host)
    }

    fn create_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()> {
        self.create_record(RecordType::A, host, record)
    }

    fn update_a_record(&self, host: &str, record: &Ipv4Addr) -> Result<()> {
        self.update_record(RecordType::A, host, record)
    }

     fn delete_a_record(&self, host: &str) -> Result<()> {
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
    use random_string::charsets::ALPHANUMERIC;

    // #[test]
    // fn test_strip_quotes() -> Result<()> {
    //     assert_eq!("abc123".to_string(), strip_quotes("\"abc123\""));
    //     assert_eq!("abc123\"", strip_quotes("abc123\""));
    //     assert_eq!("\"abc123", strip_quotes("\"abc123"));
    //     assert_eq!("abc123", strip_quotes("abc123"));

    //     Ok(())
    // }


    // pub async fn test_create_update_delete_ipv4(client: impl DnsProvider) -> Result<()> {

    //     let host = random_string::generate(16, ALPHANUMERIC);

    //     // Create
    //     let ip: Ipv4Addr = "1.1.1.1".parse()?;
    //     client.create_record(RecordType::A, &host, &ip).await?;
    //     let cur = client.get_record(RecordType::A, &host).await?;
    //     assert_eq!(Some(ip), cur);


    //     // Update
    //     let ip: Ipv4Addr = "2.2.2.2".parse()?;
    //     client.update_record(RecordType::A, &host, &ip).await?;
    //     let cur = client.get_record(RecordType::A, &host).await?;
    //     assert_eq!(Some(ip), cur);


    //     // Delete
    //     client.delete_record(RecordType::A, &host).await?;
    //     let del: Option<Ipv4Addr> = client.get_record(RecordType::A, &host).await?;
    //     assert!(del.is_none());

    //     Ok(())
    // }

    // pub async fn test_create_update_delete_txt(client: impl DnsProvider) -> Result<()> {

    //     let host = random_string::generate(16, ALPHANUMERIC);

    //     // Create
    //     let txt = "a text reference".to_string();
    //     client.create_record(RecordType::TXT, &host, &txt).await?;
    //     let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
    //     assert_eq!(txt, strip_quotes(&cur.unwrap()));


    //     // Update
    //     let txt = "another text reference".to_string();
    //     client.update_record(RecordType::TXT, &host, &txt).await?;
    //     let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
    //     assert_eq!(txt, strip_quotes(&cur.unwrap()));


    //     // Delete
    //     client.delete_record(RecordType::TXT, &host).await?;
    //     let del: Option<String> = client.get_record(RecordType::TXT, &host).await?;
    //     assert!(del.is_none());

    //     Ok(())
    // }

    // pub async fn test_create_update_delete_txt_default(client: impl DnsProvider) -> Result<()> {

    //     let host = random_string::generate(16, ALPHANUMERIC);

    //     // Create
    //     let txt = "a text reference".to_string();
    //     client.create_txt_record(&host, &txt).await?;
    //     let cur = client.get_txt_record(&host).await?;
    //     assert_eq!(txt, strip_quotes(&cur.unwrap()));


    //     // Update
    //     let txt = "another text reference".to_string();
    //     client.update_txt_record(&host, &txt).await?;
    //     let cur = client.get_txt_record(&host).await?;
    //     assert_eq!(txt, strip_quotes(&cur.unwrap()));


    //     // Delete
    //     client.delete_txt_record(&host).await?;
    //     let del = client.get_txt_record(&host).await?;
    //     assert!(del.is_none());

    //     Ok(())
    // }


}
