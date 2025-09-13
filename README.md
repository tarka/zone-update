# DNS Edit

A minimal Rust library for updating DNS records with various DNS providers.

## Overview

DNS Edit is a lightweight library that provides a simple interface for
programmatically managing DNS records. Currently, it supports the following DNS
providers:

* Gandi LiveDNS API

The library is async and offers optional for both `smol` and `tokio`.

## Features

- Support for Gandi LiveDNS API
- Optional runtime support:
  - `smol` (default) - Uses smol async runtime
  - `tokio` - Uses tokio async runtime
- Dry-run mode for testing changes before applying them

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dns-edit = "0.1.0"
```

## Usage

### Basic Example

```rust
use dns_edit::{gandi, errors::Result};
use std::net::Ipv4Addr;

async fn update_gandi_record() -> Result<()> {
    let config = dns_edit::Config {
        domain: "example.com".to_string(),
        dry_run: false,
    };
    
    let auth = gandi::Auth::ApiKey("your-api-key".to_string());
    let client = gandi::Gandi::new(config, auth);
    
    let host = "www";
    let new_ip = Ipv4Addr::new(192, 0, 2, 1);
    
    // Update the A record for www.example.com
    client.set_v4_record(host, &new_ip).await?;
    
    Ok(())
}
```


## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE-2.0.txt) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
