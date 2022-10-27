# Changelog

## [Unreleased]

### Added

- Added Listener operator to supported operators ([#143](https://github.com/stackabletech/stackablectl/pull/143))
- Support listing distributed MinIO instances ([#148](https://github.com/stackabletech/stackablectl/pull/148))

### Changed

- Bump to clap 4.0 ([#150](https://github.com/stackabletech/stackablectl/pull/150))
- Bump go modules such as helm to v3.10.1 and k8s client to v0.25.3 ([#149](https://github.com/stackabletech/stackablectl/pull/149))
- Bump kube to 0.75, k8s-openapi to 0.16 and openssl to 0.10.42 ([#154](https://github.com/stackabletech/stackablectl/pull/154))

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
