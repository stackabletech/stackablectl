# Changelog

## [Unreleased]

### Added

- Also print stack parameters when describing a demo ([#251](https://github.com/stackabletech/stackablectl/pull/251))

### Fixed

- Check HTTP status code when fetching resources via HTTP ([#258](https://github.com/stackabletech/stackablectl/pull/258))

## [0.8.0] - 2023-02-23

### Changed

- Build binaries on Ubuntu 20.04 instead of 22.04 to have glibc 2.31 instead of 2.35. This allows the binary to run on older platforms ([#235](https://github.com/stackabletech/stackablectl/pull/235))

### Fixed

- Use stable helm repo for operators ending with `-dev` ([#234](https://github.com/stackabletech/stackablectl/pull/234))

## [0.7.0] - 2023-02-14

### Added

- Support parametrization of stacks and demos ([#228](https://github.com/stackabletech/stackablectl/pull/228))
- Support listing OpenSearch Dashboards services ([#187](https://github.com/stackabletech/stackablectl/pull/187))
- Add support for listing Spark History Servers ([#210](https://github.com/stackabletech/stackablectl/pull/210))

### Changed

- Print CLI tables using [comfy-table](https://crates.io/crates/comfy-table) ([#176](https://github.com/stackabletech/stackablectl/pull/176))
- Bump kube to 0.77 ([#201](https://github.com/stackabletech/stackablectl/pull/201))
- Bump go k8s client to 0.26.0 ([#202](https://github.com/stackabletech/stackablectl/pull/202))
- Bump kube, k8s-openapi and tokio ([#205](https://github.com/stackabletech/stackablectl/pull/205))
- BREAKING: Bump format of demos and stacks to v2. Old versions of stackablectl will need an update ([#228](https://github.com/stackabletech/stackablectl/pull/228))

## [0.6.0] - 2022-10-28

### Added

- Added Listener operator to supported operators ([#143](https://github.com/stackabletech/stackablectl/pull/143))
- Support listing distributed MinIO instances ([#148](https://github.com/stackabletech/stackablectl/pull/148))

### Changed

- Bump to clap 4.0 ([#150](https://github.com/stackabletech/stackablectl/pull/150))
- Bump go modules such as helm to v3.10.1 and k8s client to v0.25.3 ([#149](https://github.com/stackabletech/stackablectl/pull/149))
- Bump kube to 0.76, k8s-openapi to 0.16 and openssl to 0.10.42 ([#154](https://github.com/stackabletech/stackablectl/pull/154))

## [0.5.0] - 2022-09-14

### Added

- Print Nifi adminuser credentials within services command ([#80](https://github.com/stackabletech/stackablectl/pull/80))

### Changed

- Sort operator versions descending ([#102](https://github.com/stackabletech/stackablectl/pull/102))

### Fixed

- Fix bug that prevents deserializing stacks and demos using `serde_yaml` 0.9 ([#103](https://github.com/stackabletech/stackablectl/pull/103))

### Removed
- Remove Spark from supported operators ([#100](https://github.com/stackabletech/stackablectl/pull/100))

## [0.4.0] - 2022-08-19

### Added

- Support demos, which are an end-to-end demonstrations of the usage of the Stackable Data Platform ([#66](https://github.com/stackabletech/stackablectl/pull/66))

## [0.3.0] - 2022-08-09

### Added

- Support stacks, which are a collection of ready-to-use Stackable data products as well as required third-party services like Postgresql or MinIO ([#36](https://github.com/stackabletech/stackablectl/pull/36))
- Add `services list` command to list the running Stackable services ([#36](https://github.com/stackabletech/stackablectl/pull/36))
- Support generation of shell completions ([#54](https://github.com/stackabletech/stackablectl/pull/54))

### Changed

- Update Rust and Go libs ([#46](https://github.com/stackabletech/stackablectl/pull/46))

## [0.2.0] - 2022-05-17

### Added

- Add option to only install subset of operators from release ([#18](https://github.com/stackabletech/stackablectl/pull/18))
- Add support to show helm release status ([#20](https://github.com/stackabletech/stackablectl/pull/20))

## [0.1.1] - 2022-05-06

### Added

- Add support for cloud vendors like Google Cloud Platform ([#12](https://github.com/stackabletech/stackablectl/pull/12))

## [0.1.0] - 2022-05-04

Initial release
