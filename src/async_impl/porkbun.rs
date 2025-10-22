
use std::sync::Arc;
use std::{fmt::Display, net::Ipv4Addr};

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::porkbun as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::porkbun::Auth;

struct Porkbun {
    inner: Arc<sync::Porkbun>,
}

impl Porkbun {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::Porkbun::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(Porkbun);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{async_impl::tests::*, generate_async_tests};
    use crate::dnsmadeeasy::tests::TEST_API;
    use std::env;

    fn get_client() -> Porkbun {
        let auth = Auth {
            key: env::var("PORKBUN_KEY").unwrap(),
            secret: env::var("PORKBUN_SECRET").unwrap(),
        };
        let config = Config {
            domain: env::var("PORKBUN_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Porkbun::new(config, auth)
    }

    generate_async_tests!("test_porkbun");

}
