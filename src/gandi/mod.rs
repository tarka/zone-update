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
        let rec: Record = match http::get(url, Some(auth)).await? {
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

        Ok(Some(rec.rrset_values[0]))

    }

    async  fn create_v4_record(&self, host: &str,ip: &Ipv4Addr) -> Result<()> {
        // PUT works for both operations
        self.update_v4_record(host, ip).await
    }

    async fn update_v4_record(&self, host: &str, ip: &Ipv4Addr) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        let update = RecordUpdate {
            rrset_values: vec![*ip],
            rrset_ttl: Some(300),
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {update:?} to {url}");
            return Ok(())
        }
        http::put::<RecordUpdate>(url, &update, Some(auth)).await?;
        Ok(())
    }

    async  fn delete_v4_record(&self,host: &str) -> Result<()> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain)
            .parse()
            .map_err(|e| Error::UrlError(format!("Error: {e}")))?;
        let auth = self.auth.get_header();

        if self.config.dry_run {
            info!("DRY-RUN: Would have sent DELETE to {url}");
            return Ok(())
        }
        http::delete(url, Some(auth)).await?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use macro_rules_attribute::apply;
    use random_string::charsets::ALPHANUMERIC;
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


    // TODO: This is generic, we could move it up to top-level testing.
    async fn test_create_update_delete_ipv4() -> Result<()> {
        let client = get_client();

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let ip = "1.1.1.1".parse()?;
        client.create_v4_record(&host, &ip).await?;
        let cur = client.get_v4_record(&host).await?;
        assert_eq!(Some(ip), cur);


        // Update
        let ip = "2.2.2.2".parse()?;
        client.update_v4_record(&host, &ip).await?;
        let cur = client.get_v4_record(&host).await?;
        assert_eq!(Some(ip), cur);


        // Delete
        client.delete_v4_record(&host).await?;
        let del = client.get_v4_record(&host).await?;
        assert!(del.is_none());

        Ok(())
    }


    #[cfg(feature = "smol")]
    mod smol {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
        async fn smol_create_update() -> Result<()> {
            test_create_update_delete_ipv4().await
        }
    }

    #[cfg(feature = "tokio")]
    mod smol {
        use super::*;

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(not(feature = "test_dnsimple"), ignore = "DnSimple API test")]
        async fn tokio_create_update() -> Result<()> {
            test_create_update_delete_ipv4().await
        }
    }


    // #[apply(test!)]
    // #[traced_test]
    // #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    // async fn test_fetch_ipv4() -> Result<()> {
    //     let client = get_client();
    //     let ip = client.get_v4_record("janus").await?;
    //     assert!(ip.is_some());
    //     assert_eq!(Ipv4Addr::new(192,168,42,1), ip.unwrap());
    //     Ok(())
    // }

    // #[apply(test!)]
    // #[traced_test]
    // #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    // async fn test_update_ipv4() -> Result<()> {
    //     let client = get_client();

    //     let cur = client.get_v4_record("test").await?
    //         .unwrap_or(Ipv4Addr::new(1,1,1,1));
    //     let next = cur.octets()[0]
    //         .wrapping_add(1);

    //     let nip = Ipv4Addr::new(next,next,next,next);
    //     client.create_v4_record("test", &nip).await?;

    //     let ip = client.get_v4_record("test").await?;
    //     if let Some(ip) = ip {
    //         assert_eq!(nip, ip);
    //     } else {
    //         assert!(false, "No updated IP found");
    //     }
    //     Ok(())
    // }

}
