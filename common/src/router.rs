//! Common utilties for serving HTTP requests.

use anyhow::Context;
use axum::Router;
use clap::Parser;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::signal::unix::{SignalKind, signal};

use crate::config::{Cli, Configuration};

/// Default address on which to listen for incoming connections.
pub(crate) fn default_listener() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 80)
}

/// Perform initialization of resources common to simulation actors and then invoke a per-actor
/// callback, then serve the resulting routes over HTTP.
pub async fn actor_main<
    C: Configuration,
    F: FnMut(C) -> Result<(&'static str, Router), anyhow::Error>,
>(
    mut callback: F,
) -> Result<(), anyhow::Error> {
    // TODO: richer tracing subscriber configuration
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config: C = C::load(&cli)?;

    let listener = tokio::net::TcpListener::bind(&config.common_configuration().listen_address)
        .await
        .context(format!(
            "failed to bind address {:?}",
            config.common_configuration().listen_address
        ))?;

    // TODO: instantiate the axum::Router here so we can plug in appropriate middleware, but for now
    // it's easier to let each actor's main() do it
    let (actor_name, routes) = callback(config)?;

    tracing::info!("started the {actor_name} simulator");

    axum::serve(listener, routes)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
