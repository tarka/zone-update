use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::dnsmadeeasy::{self as sync, API_BASE};
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::dnsmadeeasy::Auth;

/// Async wrapper around the synchronous `DnsMadeEasy` provider.
pub struct DnsMadeEasy {
    inner: Arc<sync::DnsMadeEasy>,
}

impl DnsMadeEasy {
    /// Create a new async `DnsMadeEasy` wrapper using the default endpoint.
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
    use crate::generate_async_tests;
    use crate::dnsmadeeasy::tests::TEST_API;
    use std::env;

    #[allow(unused)]
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
