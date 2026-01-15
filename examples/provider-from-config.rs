
use serde::Deserialize;
use zone_update::{Provider, errors::Result};
use std::net::Ipv4Addr;

const CONFIG_FILE: &str = "examples/provider-from-config.toml";

#[derive(Deserialize)]
pub struct MyConfig {
    domain: String,
    dry_run: bool,
    provider: Provider,
}

fn main() -> Result<()> {
    let config = std::fs::read_to_string(CONFIG_FILE)?;
    let my_config: MyConfig = toml::from_str(&config).unwrap();

    let zu_config = zone_update::Config {
        domain: my_config.domain,
        dry_run: my_config.dry_run,
    };

    let client = my_config.provider
        .blocking_impl(zu_config);

    let host = "www";
    let new_ip = Ipv4Addr::new(192, 0, 2, 1);

    client.update_a_record(host, &new_ip)?;

    Ok(())
}
