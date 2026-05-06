use std::net::SocketAddr;

use axum::{
    Router,
    routing::{get, post},
};
use clap::Parser;
use futures::FutureExt;
use snafu::{ResultExt, Whatever};
use stackable_cockpitd::{
    api_doc, handlers,
    middleware::{self, authentication::Authenticator},
};
use tokio::net::TcpListener;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt};
use utoipa_swagger_ui::SwaggerUi;

use crate::cli::Cli;

mod cli;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    let cli = Cli::parse();

    // Construct the tracing subscriber
    let format = fmt::format().with_ansi(true);
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(format)
        .pretty()
        .init();

    let authn =
        Authenticator::load_htpasswd(&cli.htpasswd).whatever_context("failed to load htpasswd")?;

    // Run the server
    let api = Router::new()
        .route("/ping", get(handlers::root::ping))
        .nest("/demos", handlers::demos::router())
        .nest("/stacks", handlers::stacks::router())
        .nest("/releases", handlers::releases::router())
        .nest("/stacklets", handlers::stacklets::router())
        .route("/login", post(middleware::authentication::log_in))
        .layer(authn.clone().layer());

    let router = Router::new()
        .nest("/api/", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc::openapi()));

    let listen_addr = SocketAddr::new(cli.address, cli.port);
    info!(addr = %listen_addr, "Starting server");

    let tcp_listener = TcpListener::bind(listen_addr)
        .await
        .whatever_context("failed to bind to listen address")?;

    if let Err(err) = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(wait_for_shutdown_signal())
        .await
    {
        eprintln!("{err}")
    }

    Ok(())
}

async fn wait_for_shutdown_signal() {
    // Copied from kube::runtime::Controller::shutdown_on_signal
    futures::future::select(
        tokio::signal::ctrl_c().map(|_| ()).boxed(),
        #[cfg(unix)]
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .map(|_| ())
            .boxed(),
        // Assume that ctrl_c is enough on non-Unix platforms (such as Windows)
        #[cfg(not(unix))]
        futures::future::pending::<()>(),
    )
    .await;
}
