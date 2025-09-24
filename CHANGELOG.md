# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/tarka/zone-edit/compare/v0.2.2...v0.2.3) - 2025-09-24

### Other

- Cleanup Cargo.toml.

## [0.2.2](https://github.com/tarka/zone-edit/compare/v0.2.1...v0.2.2) - 2025-09-24

### Other

- Change README to point to renamed project.
- spawn() shouldn't be pub

## [0.2.1](https://github.com/tarka/zone-edit/compare/v0.2.0...v0.2.1) - 2025-09-23

### Other

- Add feature flags for providers.
- Bump dependencies.

## [0.2.0](https://github.com/tarka/zone-edit/compare/v0.1.1...v0.2.0) - 2025-09-23

### Other

- Simplify address parameters.
- Use runtime-agnostic Mutex crate
- Consistent naming.
- Remove some old code.
- Add delete to gandi impl and duplicate tests from dnsimple module.
- Add record update
- Remove on-off cleanup code.
- Add delete operation and use it for cleanup.
- Expand API some more.
- Add get_v4_record() impl.
- Serde supports Ipv4Addr.
- We don't need Arc around the mutex ATM.
- Use Mutex rather than OnceLock for simplicity.
- Add ability to optionally fetch account id from upstream.
- Minor type updates.
- Rough start to dnsimple support
- Remove old code.
- Minor cleanups.
- Api tweak.
- Update Gandi impl to new http api.
- Dedup error handling.
- Simplify functions and remove some duplication.
- Debian doesn't have as many agents?
- Fix matrix config.
- Start of Github CI.
- Don't decode error body, just add it to the error message.
- Add dir-locals.el.
- Minor formatting.
- Use test domain from env.
- Add status badges.
