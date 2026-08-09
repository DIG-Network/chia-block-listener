# Changelog

All notable changes to this project are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and
[Conventional Commits](https://www.conventionalcommits.org).

## [0.4.0] - 2026-08-09

### Bug Fixes
- **peer-pool:** Require corroboration before believing a peer-announced peak (#18)

## [0.3.2] - 2026-08-09

### Bug Fixes
- **release:** Publish every crate — crates.io queries were missing a User-Agent (#20)

## [0.3.1] - 2026-08-09

### Features
- Migrate to chia-* 0.36.1, proven against live mainnet (#17)

### Bug Fixes
- **ci:** Repair the release-tag workflow, broken by an invalid TOML escape (#19)

## [0.1.19] - 2026-01-20

### Features
- Separate napi out of core rust library (initial pass)- Implement websocket driven listener

### Bug Fixes
- Update ci to properly publish dig-dns-discovery and chia-generator-parser- Implement incremental builds- Version parsing- Add dns license and description- Napi publish- Incorrect napi-rs cli version- Remove --cargo-cwd from commands that dont support it- Remove --cargo-cwd from commands that dont support it

### Chores
- Fix clippy- Fix fmt- Adjust log levels to cleaner debug output- Clippy- Bump versions

## [0.1.6] - 2025-07-14

### Features
- Index types- Puzzle reveal- Get generator data

## [0.1.4] - 2025-07-13

### Features
- Working block listener- Get historical blocks- Add sync method- Round robin peers on sync- Much faster syncing

### Chores
- **ci:** Add comprehensive GitHub Actions workflows and contribution guidelines


