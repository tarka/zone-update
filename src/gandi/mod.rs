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

use crate::{errors::{Error, Result}, http};

const API_HOST: &str = "api.gandi.net";
const API_BASE: &str = "/v5/livedns";

fn get_auth() -> Result<String> {
    let config = config::get_config()?;
    let auth = if let Some(key) = &config.gandi_api_key {
        format!("Apikey {key}")
    } else if let Some(key) = &config.gandi_pat_key {
        format!("Bearer {key}")
    } else {
        error!("No Gandi key set");
        return Err(Error::AuthError("No Gandi key set".to_string()));
    };
    Ok(auth)
}

#[allow(dead_code)]
pub async fn get_records(domain: &str) -> Result<Vec<Record>> {
    let url = format!("{API_BASE}/domains/{domain}/records");
    let recs = http::get::<Vec<Record>, types::Error>(API_HOST, &url, Some(get_auth()?)).await?
        .unwrap_or(vec![]);
    Ok(recs)
}

pub async fn get_host_ipv4(domain: &str, host: &str) -> Result<Option<Ipv4Addr>> {
    let url = format!("{API_BASE}/domains/{domain}/records/{host}/A");
    let rec: Record = match http::get::<Record, types::Error>(API_HOST, &url, Some(get_auth()?)).await? {
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

pub async fn set_host_ipv4(domain: &str, host: &str, ip: &Ipv4Addr) -> Result<()> {
    let url = format!("{API_BASE}/domains/{domain}/records/{host}/A");

    let update = RecordUpdate {
        rrset_values: vec![ip.to_string()],
        rrset_ttl: Some(300),
    };
    if config::get_config()?.dry_run.is_some_and(|b| b) {
        info!("DRY-RUN: Would have sent {update:?} to {url}");
        return Ok(())
    }
    http::put::<RecordUpdate, types::Error>(API_HOST, &url, Some(get_auth()?), &update).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use macro_rules_attribute::apply;
    use smol_macros::test;
    use temp_env::async_with_vars;
    use tracing_test::traced_test;

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_fetch_records() -> Result<()> {
        async_with_vars([("NLDDNS_CONFIG", Some("config.toml"))], async  {
            let recs = get_records("haltcondition.net").await?;
            assert!(recs.len() > 0);
            Ok(())
        }).await
    }

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_fetch_records_error() -> Result<()> {
        async_with_vars([("NLDDNS_CONFIG", Some("config.toml"))], async  {
            let recs = get_records("not.a.real.domain.net").await?;
            assert!(recs.is_empty());
            Ok(())
        }).await
    }

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_fetch_ipv4() -> Result<()> {
        async_with_vars([("NLDDNS_CONFIG", Some("config.toml"))], async  {
            let ip = get_host_ipv4("haltcondition.net", "janus").await?;
            assert!(ip.is_some());
            assert_eq!(Ipv4Addr::new(192,168,42,1), ip.unwrap());
            Ok(())
        }).await
    }

    #[apply(test!)]
    #[traced_test]
    #[cfg_attr(not(feature = "test_gandi"), ignore = "Gandi API test")]
    async fn test_update_ipv4() -> Result<()> {
        async_with_vars([("NLDDNS_CONFIG", Some("config.toml"))], async  {
            let cur = get_host_ipv4("haltcondition.net", "test").await?
                .unwrap_or(Ipv4Addr::new(1,1,1,1));
            let next = cur.octets()[0]
                .wrapping_add(1);

            let nip = Ipv4Addr::new(next,next,next,next);
            set_host_ipv4("haltcondition.net", "test", &nip).await?;

            let ip = get_host_ipv4("haltcondition.net", "test").await?;
            if let Some(ip) = ip {
                assert_eq!(nip, ip);
            } else {
                assert!(false, "No updated IP found");
            }

            Ok(())
        }).await
    }

}
