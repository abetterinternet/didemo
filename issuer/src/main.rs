use anyhow::Context;
use axum::{Json, Router, extract::State, routing::get};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio::signal::unix::{SignalKind, signal};

#[derive(Parser, Debug)]
#[command(name = "issuer", version, about)]
struct Cli {
    /// Path to configuration file.
    #[arg(long, env = "CONFIG_FILE")]
    config: PathBuf,
}

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssuerConfiguration {
    /// Address on which this server should listen for connections.
    #[serde(default = "default_listener")]
    listen_address: SocketAddr,

    /// A label identifying the issuer.
    label: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: richer tracing subscriber configuration
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config_file = File::open(cli.config).context("failed to open config file")?;

    let config: IssuerConfiguration = serde_yaml::from_reader(BufReader::new(config_file))
        .context("failed to parse config file")?;

    let listener = tokio::net::TcpListener::bind(&config.listen_address)
        .await
        .context(format!(
            "failed to bind address {:?}",
            config.listen_address
        ))?;

    let routes = Router::new()
        .route("/config", get(serve_config))
        .with_state(config);

    tracing::info!("started the issuer simulator");

    axum::serve(listener, routes)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Print the configuration.
async fn serve_config(State(config): State<IssuerConfiguration>) -> Json<IssuerConfiguration> {
    tracing::info!("serving config endpoint");
    Json(config)
}

fn default_listener() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 80)
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
