use anyhow::Context;
use axum::{Json, Router, extract::State, routing::get};
use clap::Parser;
use didemo_person::PresentedCredential;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, net::SocketAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "person", version, about)]
struct Cli {
    /// Path to configuration file.
    #[arg(long, env = "CONFIG_FILE")]
    config: PathBuf,
}

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonConfiguration {
    /// Address on which this server should listen for connections.
    listen_address: SocketAddr,

    /// The person's name.
    name: String,

    /// The person's age in years.
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: richer tracing subscriber configuration
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config_file = File::open(cli.config).context("failed to open config file")?;

    let config: PersonConfiguration = serde_yaml::from_reader(BufReader::new(config_file))
        .context("failed to parse config file")?;

    let listener = tokio::net::TcpListener::bind(&config.listen_address)
        .await
        .context(format!(
            "failed to bind address {:?}",
            config.listen_address
        ))?;

    let routes = Router::new()
        .route("/config", get(serve_config))
        .route("/present", get(present))
        .with_state(config);

    tracing::info!("started the person simulator");

    axum::serve(listener, routes).await?;

    Ok(())
}

/// Print the configuration.
async fn serve_config(State(config): State<PersonConfiguration>) -> Json<PersonConfiguration> {
    tracing::info!("serving config endpoint");
    Json(config)
}

/// Present this person's credentials.
async fn present(State(config): State<PersonConfiguration>) -> Json<PresentedCredential> {
    tracing::info!("presenting credential");
    Json(PresentedCredential {
        name: config.name,
        age: config.age,
    })
}
