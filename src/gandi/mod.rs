#![allow(unused)]

mod types;

use std::net::Ipv4Addr;
use tracing::{error, info, warn};

use types::{Record, RecordUpdate};
use crate::{errors::{Error, Result}, http, Config, DnsProvider};

const API_BASE: &str = "https://api.gandi.net/v5/livedns";

pub enum Auth {
    ApiKey(String),
    PatKey(String),
}

impl Auth {
    fn get_header(&self) -> String {
        match self {
            Auth::ApiKey(key) => format!("Apikey {key}"),
            Auth::PatKey(key) => format!("Bearer {key}"),
        }
    }
}


pub struct Gandi {
    pub config: Config,
    pub auth: Auth,
}

impl Gandi {
    pub fn new(config: Config, auth: Auth) -> Self {
        Gandi {
            config,
            auth,
        }
    }
}

impl DnsProvider for Gandi {

    async fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr>> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();
        let rec: Record = match http::get::<Record>(url, Some(auth)).await? {
            Some(rec) => rec,
            None => return Ok(None)
        };

        let nr = rec.rrset_values.len();

        // FIXME: Assumes no or single address (which probably makes sense
        // for DDNS, but may cause issues with malformed zones.
        if nr > 1 {
            error!("Returned number of IPs is {}, should be 1", nr);
            return Err(Error::UnexpectedRecord(format!("Returned number of IPs is {nr}, should be 1")));
        } else if nr == 0 {
            warn!("No IP returned for {host}, continuing");
            return Ok(None);
        }

        let ip = rec.rrset_values[0].parse()?;
        Ok(Some(ip))

    }

    async fn set_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        let update = RecordUpdate {
            rrset_values: vec![ip.to_string()],
            rrset_ttl: Some(300),
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {update:?} to {url}");
            return Ok(())
        }
        http::put::<RecordUpdate>(url, &update, Some(auth)).await?;
        Ok(())

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use macro_rules_attribute::apply;
    use smol_macros::test;
    use tracing_test::traced_test;

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

        Gandi {
            config,
            auth,
        }
    }

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_fetch_ipv4() -> Result<()> {
        let client = get_client();
        let ip = client.get_v4_record("janus").await?;
        assert!(ip.is_some());
        assert_eq!(Ipv4Addr::new(192,168,42,1), ip.unwrap());
        Ok(())
    }

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_update_ipv4() -> Result<()> {
        let client = get_client();
        let cur = client.get_v4_record("test").await?
            .unwrap_or(Ipv4Addr::new(1,1,1,1));
        let next = cur.octets()[0]
            .wrapping_add(1);

        let nip = Ipv4Addr::new(next,next,next,next);
        client.set_v4_record("test", &nip).await?;

        let ip = client.get_v4_record("test").await?;
        if let Some(ip) = ip {
            assert_eq!(nip, ip);
        } else {
            assert!(false, "No updated IP found");
        }
        Ok(())
    }

}
