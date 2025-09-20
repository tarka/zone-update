mod http;
pub mod errors;

pub mod dnsimple;
pub mod gandi;

use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::errors::Result;


pub struct Config {
    pub domain: String,
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[allow(unused)]
#[trait_variant::make(Send)]
pub trait DnsProvider {
    async fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr>>;
    async fn create_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()>;
    async fn update_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()>;
}
