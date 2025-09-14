
mod types;

use std::net::Ipv4Addr;
use tracing::{error, info, warn};

use crate::{errors::{Error, Result}, http, Config, DnsProvider};


const API_HOST: &str = "api.sandbox.dnsimple.com";
const API_BASE: &str = "/v2";


pub struct DnSimple;

impl DnSimple {

    async fn get_account_id(&self) -> Result<

}




#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use macro_rules_attribute::apply;
    use smol_macros::test;
    use tracing_test::traced_test;

    const API_HOST: &str = "api.sandbox.dnsimple.com";

    fn get_client() -> DnSimple {
        DnSimple {}
    }

}
