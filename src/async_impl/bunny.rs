
use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::bunny as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::bunny::Auth;

pub struct Bunny {
    inner: Arc<sync::Bunny>,
}

impl Bunny {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::Bunny::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(Bunny);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    #[allow(unused)]
    fn get_client() -> Bunny {
        let auth = Auth {
            key: env::var("BUNNY_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("BUNNY_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Bunny::new(config, auth)
    }

    generate_async_tests!("test_bunny");

}
