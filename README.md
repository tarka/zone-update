# Zone Update

[![Crates.io](https://img.shields.io/crates/v/zone-update)](https://crates.io/crates/zone-update)
[![Docs.rs](https://docs.rs/zone-update/badge.svg)](https://docs.rs/zone-update)
[![GitHub CI](https://github.com/tarka/zone-update/actions/workflows/tests.yml/badge.svg)](https://github.com/tarka/zone-update/actions)
[![License](https://img.shields.io/crates/l/zone-update)](https://github.com/tarka/zone-update/blob/main/README.md#License)

A minimal Rust library for updating DNS records with various DNS providers.

## Overview

Zone Update is a lightweight library that provides a simple interface for
programmatically managing DNS records through provider APIs.
The library is async and supports both `smol` and `tokio`.

### Supported DNS providers

Currently, Zone Update supports the following DNS providers:

* Gandi
* Dnsimple
* DnsMadeEasy
* Porkbun

See the [DNS providers matrix](docs/PROVIDERS.md) for more details.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zone-update = "0.1.0"
```

## Usage

### Basic Example

```rust
use zone_update::{gandi, errors::Result};
use std::net::Ipv4Addr;

async fn update_gandi_record() -> Result<()> {
    let config = zone_update::Config {
        domain: "example.com".to_string(),
        dry_run: false,
    };
    
    let auth = gandi::Auth::ApiKey("your-api-key".to_string());
    let client = gandi::Gandi::new(config, auth);
    
    let host = "www";
    let new_ip = Ipv4Addr::new(192, 0, 2, 1);

    // Update the A record for www.example.com
    client.update_v4_record(host, &new_ip).await?;
    
    Ok(())
}
```

See the `examples` directory for other use-cases.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE-2.0.txt) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
