use anyhow::Context;
use axum::{Json, Router, extract::State, routing::get};
use clap::Parser;
use didemo_wallet::CredentialType;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, net::SocketAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "wallet", version, about)]
struct Cli {
    /// Path to configuration file.
    #[arg(long, env = "CONFIG_FILE")]
    config: PathBuf,
}

/// Configuration for a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletConfiguration {
    /// Address on which this server should listen for connections.
    listen_address: SocketAddr,

    /// The wallet vendor's name.
    vendor: String,

    /// Initial credentials in this wallet.
    initial_credentials: HashMap<CredentialType, String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: richer tracing subscriber configuration
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config_file = File::open(cli.config).context("failed to open config file")?;

    let config: WalletConfiguration = serde_yaml::from_reader(BufReader::new(config_file))
        .context("failed to parse config file")?;

    let listener = tokio::net::TcpListener::bind(&config.listen_address)
        .await
        .context(format!(
            "failed to bind address {:?}",
            config.listen_address
        ))?;

    let routes = Router::new()
        .route("/config", get(serve_config))
        .route("/credentials", get(credentials))
        // TODO: route for adding a credential to wallet
        //.route("/add-credential", put(???))
        .with_state(config);

    tracing::info!("started the wallet simulator");

    axum::serve(listener, routes).await?;

    Ok(())
}

/// Print the configuration.
async fn serve_config(State(config): State<WalletConfiguration>) -> Json<WalletConfiguration> {
    tracing::info!("serving config endpoint");
    Json(config)
}

/// Print all the credentials stored in the wallet.
async fn credentials(
    State(config): State<WalletConfiguration>,
) -> Json<HashMap<CredentialType, String>> {
    tracing::info!("serving credentials endpoint");
    Json(config.initial_credentials)
}
