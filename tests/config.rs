
use std::fs::read_to_string;

use anyhow::{Context, Result};
use serde::Deserialize;

use zone_update::Providers;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Ddns {
    pub domain: String,
    pub host: String,
    pub provider: Providers,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub log_level: Option<String>,
    pub iface: String,
    pub ddns: Ddns,
    #[serde(default)]
    pub dry_run: bool,
}

#[test]
fn test_config() -> Result<()> {
    let confile = "tests/config.corn";
    let conf_s = read_to_string(confile)
        .with_context(|| format!("Failed to load config from {confile}"))?;

    let conf = corn::from_str::<Config>(&conf_s)?;

    match conf.ddns.provider {
        Providers::PorkBun(auth) => {
            assert_eq!("a_key".to_string(), auth.key);
            assert_eq!("a_secret".to_string(), auth.secret);
        }
        _ => panic!("Didn't match provider")
    }

    Ok(())
}

