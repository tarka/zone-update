# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/tarka/zone-update/compare/v0.5.0...v0.5.1) - 2025-10-28

### Other

- Change the default trait fns to abstract, but generate the boilerplate in a macro. This removes all the Sized/Box/dyn complexity.
- Cleanup some warnings
- Make dyn compatible.

## [0.5.0](https://github.com/tarka/zone-update/compare/v0.4.2...v0.5.0) - 2025-10-26

### Other

- Simplify Dnsimple name
- Add docs to DnsProvider

## [0.4.2](https://github.com/tarka/zone-update/compare/v0.4.1...v0.4.2) - 2025-10-25

### Other

- Correct a provider.
- Fill in a few more providers.
- Update dynadot note.
- Fix error in imports.
- Cleanup imports.
- Updated docs for check_error
- Re-add some missing error handling.
- Add missing pub exports.
- Missed tokio
- Fix import of async tests.
- Note about dynadot.
- Lots of cleanups of unused

## [0.4.1](https://github.com/tarka/zone-update/compare/v0.4.0...v0.4.1) - 2025-10-22

### Other

- Limite per-provider tests to one at a time.
- Use single thread for tests against sandboxes.
- Update list of supported providers.
- Add async impl for porkbun.
- Fixes for porkbun.
- Initial porkbun support, still needs some fixes to decoding.
- Re-export Auth from async modules for convenience.

## [0.4.0](https://github.com/tarka/zone-update/compare/v0.3.1...v0.4.0) - 2025-10-21

### Other

- Update provider list.
- Actually add dnsmadeeasy
- Add dnsmadeeasy to CI.
- Add DnsMadeEasy impl, plus misc. fixes for problems found along the way.
- More automated docs for http.rs
- Add a couple of more ureq helpers.
- Replace unwrap with error
- Add more docs to http.rs.
- Add automated docs to WithHeader extension trait.
- Start adding dnsmadeeasy
- Fix feature name
- Use same provider names as in sync version.
- Fix macro name-clash.
- Scope some tests
- Also generate boilerplate for sync testing. Add auto-generated docs for macros.
- Fix feature name
- Correct actions syntax.
- Enable dnsimple sandbox testing in CI.
- More small cleanups.
- Convert some async boilerplate to macros and add gandi async impl.
- Convert boiler-plate async impl to macro
- Minor cleanups
- Remove old async test code.
- Initial implementation of async API using `blocking`.
- Convert Gandi to ureq.
- Move common utils into http.rs
- Intial working and tests for dnsimple.
- Start of converting dnsimple to ureq.
- Note about Namecheap sandbox payment.
- Update DnsMadeEasy sandbox status
- More cleanups.
- Remove dnsmadeeasy code prior to merge of other changes to master.
- More DnsMadeEasy changes.
- Default auth header doesn't need to be Option anymore.
- Refactor http module to allow custom headers.
- Interim checkin prior to http.rs changes.
- Add skeleton of DnsMadeEasy impl.
- Move resuable API tests up to root.
- Minor cleanups to dnsimple.
- More readme tweaks.
- Minor readme update.
- Provider corrections.
- Fix link
- Move provider matrix to own file and expand from acme.sh list.
- Clarify note
- Use emojis
- Start of DNS provider list.
- Add note about Gandi keys.
- Rename example.
- Dump cert and private key in example.
- The example needs gandi to build.
- Cleanups and clarifications for example.
- Add practical example using acme-micro to create an cert with letsencrypt.
- In the wild we need to specify a crypto provider to rustls.
- Minor cleanups.

## [0.3.1](https://github.com/tarka/zone-update/compare/v0.3.0...v0.3.1) - 2025-09-26

### Other

- Add helpers for A records

## [0.3.0](https://github.com/tarka/zone-update/compare/v0.2.3...v0.3.0) - 2025-09-26

### Other

- Add helper functions for common operations.
- Fix gandi record type.
- Add support and testing for TXT types.
- Move API entirely to generics
- Start moving to multiple record types
- Use async OnceCell/Arc for root store to save regenerating it.
- Add a note about using pollster+thread for fallback.
- Minor import cleanups.
- Fix base URL.
- Fix CI badge.

## [0.2.3](https://github.com/tarka/zone-update/compare/v0.2.2...v0.2.3) - 2025-09-24

### Other

- Cleanup Cargo.toml.

## [0.2.2](https://github.com/tarka/zone-update/compare/v0.2.1...v0.2.2) - 2025-09-24

### Other

- Change README to point to renamed project.
- spawn() shouldn't be pub

## [0.2.1](https://github.com/tarka/zone-update/compare/v0.2.0...v0.2.1) - 2025-09-23

### Other

- Add feature flags for providers.
- Bump dependencies.

## [0.2.0](https://github.com/tarka/zone-update/compare/v0.1.1...v0.2.0) - 2025-09-23

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
