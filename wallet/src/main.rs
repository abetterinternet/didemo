use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    credential::Credential,
    router::{AppError, actor_main},
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Configuration for a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// The wallet vendor's name.
    vendor: String,
}

impl Configuration for WalletConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

#[derive(Clone, Debug)]
struct Wallet {
    config: WalletConfiguration,
    _http_client: Client,
    credentials: Vec<Credential>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config, client_builder| {
        let _http_client = client_builder.build()?;

        let wallet = Wallet {
            config,
            _http_client,
            // TODO: load credentials from persistent storage
            credentials: Vec::new(),
        };

        let routes = Router::new()
            .route("/config", get(serve_config))
            .route("/credentials", get(credentials))
            .route("/credentials", put(store_credential))
            .with_state(Arc::new(Mutex::new(wallet)));

        Ok(("wallet".to_string(), routes))
    })
    .await
}

/// Print the configuration.
async fn serve_config(State(wallet): State<Arc<Mutex<Wallet>>>) -> Json<WalletConfiguration> {
    tracing::info!("serving config endpoint");
    Json(wallet.lock().unwrap().config.clone())
}

/// Print all the credentials stored in the wallet.
async fn credentials(State(wallet): State<Arc<Mutex<Wallet>>>) -> Json<Vec<Credential>> {
    tracing::info!("serving credentials endpoint");
    Json(wallet.lock().unwrap().credentials.clone())
}

/// Store the credential in the wallet.
async fn store_credential(
    State(wallet): State<Arc<Mutex<Wallet>>>,
    Json(request): Json<Credential>,
) -> Result<StatusCode, AppError> {
    // TODO: policy checks? For uniqueness on certain keys?

    // TODO: Verify that issuer is trusted? Or is that not something wallets should do?

    // TODO: cryptography: check signature somehow?

    wallet.lock().unwrap().credentials.push(request);

    Ok(StatusCode::CREATED)
}
