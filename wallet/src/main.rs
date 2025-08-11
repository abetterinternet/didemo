use axum::{Json, Router, extract::State, routing::get};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    router::actor_main,
};
use didemo_wallet::CredentialType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// The wallet vendor's name.
    vendor: String,

    /// Initial credentials in this wallet.
    initial_credentials: HashMap<CredentialType, String>,
}

impl Configuration for WalletConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config| {
        let routes = Router::new()
            .route("/config", get(serve_config))
            .route("/credentials", get(credentials))
            // TODO: route for adding a credential to wallet
            //.route("/add-credential", put(???))
            .with_state(config);

        Ok(("wallet", routes))
    })
    .await
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
