
use std::sync::Arc;
use std::{fmt::Display, net::Ipv4Addr};

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::dnsmadeeasy::{self as sync, API_BASE};
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::dnsmadeeasy::Auth;

struct DnsMadeEasy {
    inner: Arc<sync::DnsMadeEasy>,
}

impl DnsMadeEasy {
    pub fn new(config: Config, auth: Auth) -> Self {
        Self::new_with_endpoint(config, auth, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, endpoint: &'static str) -> Self {
        let inner = sync::DnsMadeEasy::new_with_endpoint(config, auth, endpoint);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(DnsMadeEasy);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{async_impl::tests::*, generate_async_tests};
    use crate::dnsmadeeasy::tests::TEST_API;
    use std::env;

    fn get_client() -> DnsMadeEasy {
        let auth = Auth {
            key: env::var("DNSMADEEASY_KEY").unwrap(),
            secret: env::var("DNSMADEEASY_SECRET").unwrap(),
        };
        let config = Config {
            domain: env::var("DNSMADEEASY_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DnsMadeEasy::new_with_endpoint(config, auth, TEST_API)
    }

    generate_async_tests!("test_dnsmadeeasy");

}
