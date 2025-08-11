//! Configuration options common to simulation actors.

use crate::router::default_listener;
use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fs::File, io::BufReader, net::SocketAddr, path::PathBuf};

/// Command line options for simulation actors.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// Path to configuration file.
    #[arg(long, env = "CONFIG_FILE")]
    pub(crate) config: PathBuf,
}

/// Configuration file items common to simulation actors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfiguration {
    /// Address on which this server should listen for connections.
    #[serde(default = "default_listener")]
    pub(crate) listen_address: SocketAddr,
}

pub trait Configuration: DeserializeOwned {
    fn common_configuration(&self) -> &CommonConfiguration;

    fn load(cli: &Cli) -> Result<Self, anyhow::Error> {
        let config_file = File::open(&cli.config).context("failed to open config file")?;

        serde_yaml::from_reader(BufReader::new(config_file)).context("failed to parse config file")
    }
}
