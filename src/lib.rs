mod http;
pub mod errors;


#[cfg(feature = "dnsimple")]
pub mod dnsimple;
#[cfg(feature = "gandi")]
pub mod gandi;

use std::{fmt::{self, Debug, Display, Formatter}};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

    async fn delete_record(&self, rtype: RecordType, host: &str) -> Result<()>
;

}
