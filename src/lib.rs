mod http;
pub mod errors;


#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "gandi")]
pub mod gandi;

use std::{fmt::{self, Debug, Display, Formatter}};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::errors::{Error, Result};


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
#[trait_variant::make(Send)]
pub trait DnsProvider {
    async fn get_record<T>(&self, rtype: RecordType, host: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned;

    async fn create_record<T>(&self, rtype: RecordType, host: &str, ip: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    async fn update_record<T>(&self, rtype: RecordType, host: &str, ip: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync;

    async fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>;

}


fn strip_quotes(record: &str) -> Result<String> {
    let chars = record.chars();
    let mut check = chars.clone();

    let first = check.nth(0)
        .ok_or(Error::UnexpectedRecord("Empty string".to_string()))?;
    let last = check.last()
        .ok_or(Error::UnexpectedRecord("Empty string".to_string()))?;

    if first != '"' || last != '"' {
        return Err(Error::UnexpectedRecord("Quotes not found".to_string()));
    }

    let stripped = chars.skip(1)
        .take(record.len() - 2)
        .collect();

    Ok(stripped)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_quotes() -> Result<()> {
        assert_eq!("abc123".to_string(), strip_quotes("\"abc123\"")?);
        assert!(strip_quotes("abc123\"").is_err());
        assert!(strip_quotes("\"abc123").is_err());

        Ok(())
    }


}
