
use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::desec as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::desec::Auth;

pub struct DeSec {
    inner: Arc<sync::DeSec>,
}

impl DeSec {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::DeSec::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(DeSec);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    #[allow(unused)]
    fn get_client() -> DeSec {
        let auth = Auth {
            key: env::var("DESEC_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("DESEC_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        DeSec::new(config, auth)
    }

    generate_async_tests!("test_desec");

}
