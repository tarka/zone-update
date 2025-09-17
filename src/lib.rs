mod http;
pub mod errors;

//pub mod gandi;

use std::net::Ipv4Addr;

use crate::errors::Result;


pub struct Config {
    pub domain: String,
    pub dry_run: bool,
}

#[allow(unused)]
#[trait_variant::make(Send)]
pub trait DnsProvider {
    async fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr>>;
    async fn set_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()>;
}
