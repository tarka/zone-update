#![allow(unused)]

// dns-edit: DNS provider update utilities
// Copyright (C) 2025 tarkasteve@gmail.com
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod types;

use std::net::Ipv4Addr;

use tracing::{error, info, warn};

use types::{Record, RecordUpdate};

use crate::{errors::{Error, Result}, http, Config, DnsProvider};

const API_HOST: &str = "api.gandi.net";
const API_BASE: &str = "/v5/livedns";

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


struct Gandi {
    config: Config,
    auth: Auth,
}

impl Gandi {
    fn new(config: Config, auth: Auth) -> Self {
        Gandi {
            config,
            auth,
        }
    }
}

impl DnsProvider for Gandi {

    async fn get_v4_record(&self, host: &str) -> Result<Option<Ipv4Addr>> {
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain);
        let auth = self.auth.get_header();
        let rec: Record = match http::get::<Record, types::Error>(API_HOST, &url, Some(auth)).await? {
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
        let url = format!("{API_BASE}/domains/{}/records/{host}/A", self.config.domain);
        let auth = self.auth.get_header();

        let update = RecordUpdate {
            rrset_values: vec![ip.to_string()],
            rrset_ttl: Some(300),
        };
        if self.config.dry_run {
            info!("DRY-RUN: Would have sent {update:?} to {url}");
            return Ok(())
        }
        http::put::<RecordUpdate, types::Error>(API_HOST, &url, Some(auth), &update).await?;
        Ok(())

    }

}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
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
            domain: "haltcondition.net".to_string(),
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
