# stackable-cockpitd

## The stack

Like all our software, the `stackable-cockpitd` server is written in Rust. The server internally uses the `stackable-cockpit` library
and exposes functionality via an HTTP REST API. Core community-provided dependencies are:

- [`tokio`][tokio-link], the async runtime, which ised used to handle incoming network requests over HTTP/TCP.
- [`axum`][axum-link], a HTTP framework for building REST APIs. It is based on the `tokio` runtime.
- [`utoipa`][utoipa-link], a proc-macro/derive-based OpenAPI spec generator that nicely integrates with `axum`.
- [`clap`][clap-link], a powerful command line argument parser that powers the CLI interface.
- [`tracing`][tracing-link] is used for context-rich application logging. Our complete stack emits tracing events that can be consumed via multiple different subscribers.
- [`snafu`][snafu-link] for easy custom error handling.

## Development

```shell
cargo run -p stackable-cockpitd -- <ARGS>
```

```shell
cargo build -p stackable-cockpitd --release # or --debug
./target/release/stackable-cockpitd <ARGS>
```

[tokio-link]: https://tokio.rs/
[axum-link]: https://github.com/tokio-rs/axum
[utoipa-link]: https://github.com/juhaku/utoipa
[clap-link]: https://github.com/clap-rs/clap
[tracing-link]: https://github.com/tokio-rs/tracing
[snafu-link]: https://github.com/shepmaster/snafu
