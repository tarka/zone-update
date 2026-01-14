
use std::sync::Arc;
use std::fmt::Display;

use blocking::unblock;
use serde::{de::DeserializeOwned, Serialize};

use crate::linode as sync;
use crate::{async_provider_impl, Config, DnsProvider};
use crate::{errors::Result, RecordType};

use crate::async_impl::AsyncDnsProvider;


pub use crate::linode::Auth;

pub struct Linode {
    inner: Arc<sync::Linode>,
}

impl Linode {
    pub fn new(config: Config, auth: Auth) -> Self {
        let inner = sync::Linode::new(config, auth);
        Self {
            inner: Arc::new(inner)
        }
    }
}

async_provider_impl!(Linode);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_async_tests;
    use std::env;

    #[allow(unused)]
    fn get_client() -> Linode {
        let auth = Auth {
            key: env::var("LINODE_API_KEY").unwrap(),
        };
        let config = Config {
            domain: env::var("LINODE_TEST_DOMAIN").unwrap(),
            dry_run: false,
        };
        Linode::new(config, auth)
    }

    generate_async_tests!("test_linode");

}
