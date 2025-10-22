
use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::gandi as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;

pub use crate::gandi::Auth;

pub struct Gandi {
    inner: Arc<sync::Gandi>,
}

impl Gandi {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::Gandi::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }

}

async_provider_impl!(Gandi);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    fn get_client() -> Gandi {
        let auth = if let Some(key) = env::var("GANDI_APIKEY").ok() {
            Auth::ApiKey(key)
        } else if let Some(key) = env::var("GANDI_PATKEY").ok() {
            Auth::PatKey(key)
        } else {
            panic!("No Gandi auth key set");
        };

        let config = Config {
            domain: env::var("GANDI_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };

        Gandi::new(config, auth)
    }

    generate_async_tests!("test_gandi");

}

