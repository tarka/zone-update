use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::cloudflare as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::cloudflare::Auth;

/// Async wrapper around the synchronous `Cloudflare` provider.
pub struct Cloudflare {
    inner: Arc<sync::Cloudflare>,
}

impl Cloudflare {
    /// Create a new async `Cloudflare` wrapper.
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::Cloudflare::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(Cloudflare);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    #[allow(unused)]
    fn get_client() -> Cloudflare {
        let auth = Auth {
            key: env::var("CLOUDFLARE_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("CLOUDFLARE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Cloudflare::new(config, auth)
    }

    generate_async_tests!("test_cloudflare");
}
