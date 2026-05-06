<!-- markdownlint-disable MD041 MD033 -->

<p align="center">
  <img width="150" src="./.readme/static/borrowed/Icon_Stackable.svg" alt="Stackable Logo"/>
</p>

<h1 align="center">Stackable Cockpit</h1>

[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-green.svg)](https://docs.stackable.tech/home/stable/contributor/index.html)
[![Apache License 2.0](https://img.shields.io/badge/license-Apache--2.0-green)](./LICENSE)

[Stackable Data Platform](https://stackable.tech/) | [Platform Docs](https://docs.stackable.tech/) | [Discussions](https://github.com/orgs/stackabletech/discussions) | [Discord](https://discord.gg/7kZ3BNnCAF)

This repository contains the Stackable library `stackable-cockpit`, the Stackable CLI tool `stackablectl`, and the
Stackable Cockpit server `stackable-cockpitd`.

## Components

- [`stackable-cockpit`][lib-readme]: The underlying library for all actions related to the Stackable Data Platform
- [`stackablectl`][ctl-readme]: CLI tool to interact with local and remote deployments of the data platform
- [`stackable-cockpitd`][server-readme]: API server used by frontends to interact with the data platform

## Developer Setup

### Prerequisites

- A working (and up2date) Rust installation including rustc, clippy, and cargo
- Optional, but strongly advised: a working [pre-commit][pre-commit] installation

### Getting started

```shell
git clone git@github.com:stackabletech/stackable-cockpit.git
cd stackable-cockpit
```

Each component can be build separately like this:

```shell
cargo build --release -p stackablectl       # Builds stackablectl
cargo build --release -p stackable-cockpitd # Builds the Stackable Cockpit API server
```

### Pre-commit hooks and xtasks

This repository uses multiple pre-commit hooks to run checks, formatting and code-generation on different files. The
hooks are:

- [`trailing-whitespace`](https://github.com/pre-commit/pre-commit-hooks#trailing-whitespace): Trims trailing whitespace
  in all files
- [`end-of-file-fixer`](https://github.com/pre-commit/pre-commit-hooks#end-of-file-fixer): Files need to end with
  newlines
- [`detect-aws-credentials`](https://github.com/pre-commit/pre-commit-hooks#detect-aws-credentials): Detect AWS secrets
- [`detect-private-key`](https://github.com/pre-commit/pre-commit-hooks#detect-private-key): Detect private keys
- [`yamllint`](https://github.com/adrienverge/yamllint): Runs linting on all YAML files
- [`markdownlint`](https://github.com/igorshubovych/markdownlint-cli): Runs linting on all Markdown files
- `cargo clippy --all-targets --all-features -- -D warnings`: Runs Clippy on all files and errors on warnings
- `cargo fmt -- --check`: Checks if Rust code needs formatting
- `cargo xtask gen-comp`: Runs shell completions generation for `stackablectl`
- `cargo xtask gen-man`: Runs man page generation for `stackablectl`
- `cargo xtask gen-ctl-readme`: Generates and inserts `stackablectl` help text into README

[server-readme]: ./rust/stackable-cockpitd/README.md
[ctl-readme]: ./rust/stackablectl/README.md
[pre-commit]: https://pre-commit.com/
[lib-readme]: ./rust/stackable-cockpit/README.md

### Templating variables

| Variable    | Availability                                                                            | Content                                                                       |
| ----------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `NAMESPACE` | Always                                                                                  | The namespace where the stack and demo (not the operators!) are deployed into |
| `STACK`     | Always (both in stack and demo manifests)                                               | The name of the stack                                                         |
| `DEMO`      | In demos manifests: Always<br>In stack manifests: Only when deployed as part of a demo! | The name of the demo                                                          |
