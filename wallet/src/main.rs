use anyhow::{Context, anyhow};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use didemo_common::{
    bbs::BbsKeypair,
    config::{CommonConfiguration, Configuration},
    credential::{Credential, CredentialType, DriversLicense, LibraryCard},
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

    let messages = match request.credential_type {
        CredentialType::LibraryCard => {
            let decoded_credential: LibraryCard =
                serde_json::from_str(&request.encoded_credential).unwrap();

            Vec::from([
                decoded_credential.library_name.into_bytes(),
                decoded_credential.holder_name.into_bytes(),
                decoded_credential.serial_number.to_be_bytes().to_vec(),
            ])
        }
        CredentialType::DriversLicense => {
            let decoded_credential: DriversLicense =
                serde_json::from_str(&request.encoded_credential).unwrap();

            Vec::from([
                decoded_credential.issuing_jurisdiction.into_bytes(),
                decoded_credential.holder_name.into_bytes(),
                decoded_credential.serial_number.to_be_bytes().to_vec(),
                decoded_credential.home_address.into_bytes(),
                if decoded_credential.organ_donor {
                    Vec::from([1])
                } else {
                    Vec::from([0])
                },
                decoded_credential.birthdate.to_be_bytes().to_vec(),
            ])
        }
    };

    // TODO: Verify that issuer is trusted? For now we just derive the keys based on the BBS
    // signature header.
    let issuer_keypair = BbsKeypair::new(
        str::from_utf8(&request.signature.header)
            .context("failed to convert BBS header to issuer name")?,
    )?;

    if !issuer_keypair.verify(
        request.signature.header.clone(),
        messages,
        request.signature.signature.clone(),
    )? {
        return Err(anyhow!("invalid signature on incoming credential").into());
    }

    wallet.lock().unwrap().credentials.push(request);

    Ok(StatusCode::CREATED)
}
