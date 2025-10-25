use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::dnsimple::{self as sync, API_BASE};
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::dnsimple::Auth;

pub struct DnSimple {
    inner: Arc<sync::DnSimple>,
}

impl DnSimple {
    pub fn new(config: Config, auth: Auth, acc: Option<u32>) -> Self {
        Self::new_with_endpoint(config, auth, acc, API_BASE)
    }

    fn new_with_endpoint(config: Config, auth: Auth, acc: Option<u32>, endpoint: &'static str) -> Self {
        let inner = sync::DnSimple::new_with_endpoint(config, auth, acc, endpoint);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(DnSimple);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    const TEST_API: &str = "https://api.sandbox.dnsimple.com/v2";

    fn get_client() -> DnSimple {
        let auth = Auth { key: env::var("DNSIMPLE_TOKEN").unwrap() };
        let config = Config {
            domain: env::var("DNSIMPLE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DnSimple::new_with_endpoint(config, auth, None, TEST_API)
    }

    generate_async_tests!("test_dnsimple");

}
