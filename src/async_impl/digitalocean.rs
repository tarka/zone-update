
use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::digitalocean as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::digitalocean::Auth;

pub struct DigitalOcean {
    inner: Arc<sync::DigitalOcean>,
}

impl DigitalOcean {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::DigitalOcean::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(DigitalOcean);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    #[allow(unused)]
    fn get_client() -> DigitalOcean {
        let auth = Auth {
            key: env::var("DIGITALOCEAN_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("DIGITALOCEAN_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DigitalOcean::new(config, auth)
    }

    generate_async_tests!("test_digitalocean");

}
