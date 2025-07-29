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
    messages::person::{Proof, ProofRequest, ProofType},
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
            .route("/proof", get(prove))
            .with_state(Arc::new(Mutex::new(wallet)));

        Ok(("wallet".to_string(), routes))
    })
    .await
}

/// Print the configuration.
async fn serve_config(State(wallet): State<Arc<Mutex<Wallet>>>) -> Json<WalletConfiguration> {
    Json(wallet.lock().unwrap().config.clone())
}

/// Print all the credentials stored in the wallet.
async fn credentials(State(wallet): State<Arc<Mutex<Wallet>>>) -> Json<Vec<Credential>> {
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
            let decoded_credential: LibraryCard = serde_json::from_str(&request.encoded_credential)
                .context("failed to decode library card")?;

            Vec::from([
                decoded_credential.library_name.into_bytes(),
                decoded_credential.holder_name.into_bytes(),
                decoded_credential.serial_number.to_be_bytes().to_vec(),
            ])
        }
        CredentialType::DriversLicense => {
            let decoded_credential: DriversLicense =
                serde_json::from_str(&request.encoded_credential)
                    .context("failed to decode driver's license")?;

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

    // Verify the BBS signature (not any proof on any message!) just for kicks.
    issuer_keypair.verify(
        request.signature.header.clone(),
        messages,
        request.signature.signature.clone(),
    )?;

    wallet.lock().unwrap().credentials.push(request);

    Ok(StatusCode::CREATED)
}

/// Prove to a verifier that a message is signed.
#[axum::debug_handler]
async fn prove(
    State(wallet): State<Arc<Mutex<Wallet>>>,
    Json(proof_request): Json<ProofRequest>,
) -> Result<Json<Proof>, AppError> {
    let wallet = wallet.lock().unwrap();

    tracing::info!(proof_type = ?proof_request.proof_type, "proving credential attribute");

    match proof_request.proof_type {
        ProofType::HoldsDriversLicense => {
            for credential in &wallet.credentials {
                if credential.credential_type == CredentialType::DriversLicense {
                    let issuer_keypair = BbsKeypair::new(
                        str::from_utf8(&credential.signature.header)
                            .context("failed to convert BBS header to issuer name")?,
                    )?;

                    let decoded_credential: DriversLicense =
                        serde_json::from_str(&credential.encoded_credential)
                            .context("failed to decode driver's license")?;

                    // Disclose no messages, only proof that the holder holds *some* license issued
                    // by the issuer. However the proof algorithm still needs all the messages.
                    let messages = Vec::from([
                        (false, decoded_credential.issuing_jurisdiction.into_bytes()),
                        (false, decoded_credential.holder_name.into_bytes()),
                        (
                            false,
                            decoded_credential.serial_number.to_be_bytes().to_vec(),
                        ),
                        (false, decoded_credential.home_address.into_bytes()),
                        (
                            false,
                            if decoded_credential.organ_donor {
                                Vec::from([1])
                            } else {
                                Vec::from([0])
                            },
                        ),
                        (false, decoded_credential.birthdate.to_be_bytes().to_vec()),
                    ]);

                    let proof = issuer_keypair.prove(
                        credential.signature.header.clone(),
                        messages,
                        credential.signature.signature.clone(),
                    )?;

                    if let Err(error) = issuer_keypair.verify_proof(
                        credential.signature.header.clone(),
                        Vec::new(),
                        proof.clone(),
                    ) {
                        tracing::info!("failed to verify DL hold proof: {error:?}");
                    }

                    return Ok(Json(Proof {
                        header: credential.signature.header.clone(),
                        proof,
                        disclosed_messages: Vec::new(),
                    }));
                }
            }
            Err(anyhow!("found no driver's license in wallet").into())
        }
        ProofType::HoldsLibraryCard => {
            for credential in &wallet.credentials {
                if credential.credential_type == CredentialType::LibraryCard {
                    let issuer_keypair = BbsKeypair::new(
                        str::from_utf8(&credential.signature.header)
                            .context("failed to convert BBS header to issuer name")?,
                    )?;

                    let decoded_credential: LibraryCard =
                        serde_json::from_str(&credential.encoded_credential)
                            .context("failed to decode library card")?;

                    // Disclose no messages, only proof that the holder holds *some* license issued
                    // by the issuer. However the proof algorithm still needs all the messages.
                    let messages = Vec::from([
                        (false, decoded_credential.library_name.into_bytes()),
                        (false, decoded_credential.holder_name.into_bytes()),
                        (
                            false,
                            decoded_credential.serial_number.to_be_bytes().to_vec(),
                        ),
                    ]);
                    let proof = issuer_keypair.prove(
                        credential.signature.header.clone(),
                        messages,
                        credential.signature.signature.clone(),
                    )?;

                    if let Err(error) = issuer_keypair.verify_proof(
                        credential.signature.header.clone(),
                        Vec::new(),
                        proof.clone(),
                    ) {
                        tracing::info!("failed to verify library card hold proof: {error:?}");
                    }

                    return Ok(Json(Proof {
                        header: credential.signature.header.clone(),
                        proof,
                        disclosed_messages: Vec::new(),
                    }));
                }
            }
            Err(anyhow!("found no library card in wallet").into())
        }
        ProofType::HolderName => {
            // Either credential has the name in it, but we'll hard code the driver's license for
            // now.
            for credential in &wallet.credentials {
                if credential.credential_type == CredentialType::DriversLicense {
                    let issuer_keypair = BbsKeypair::new(
                        str::from_utf8(&credential.signature.header)
                            .context("failed to convert BBS header to issuer name")?,
                    )?;

                    let decoded_credential: DriversLicense =
                        serde_json::from_str(&credential.encoded_credential)
                            .context("failed to decode driver's license")?;

                    // Disclose only the holder name message in the proof.
                    let messages = Vec::from([
                        (false, decoded_credential.issuing_jurisdiction.into_bytes()),
                        (true, decoded_credential.holder_name.clone().into_bytes()),
                        (
                            false,
                            decoded_credential.serial_number.to_be_bytes().to_vec(),
                        ),
                        (false, decoded_credential.home_address.into_bytes()),
                        (
                            false,
                            if decoded_credential.organ_donor {
                                Vec::from([1])
                            } else {
                                Vec::from([0])
                            },
                        ),
                        (false, decoded_credential.birthdate.to_be_bytes().to_vec()),
                    ]);

                    let proof = issuer_keypair.prove(
                        credential.signature.header.clone(),
                        messages.clone(),
                        credential.signature.signature.clone(),
                    )?;

                    let disclosed_messages_with_index =
                    // Zero based index of the name in the driver's license happens to be 1
                        Vec::from([(1, decoded_credential.holder_name.into_bytes())]);

                    if let Err(error) = issuer_keypair.verify_proof(
                        credential.signature.header.clone(),
                        disclosed_messages_with_index.clone(),
                        proof.clone(),
                    ) {
                        tracing::info!("failed to verify holder name proof: {error:?}");
                    }

                    return Ok(Json(Proof {
                        header: credential.signature.header.clone(),
                        proof,
                        disclosed_messages: disclosed_messages_with_index,
                    }));
                }
            }
            Err(anyhow!("found no driver's license in wallet").into())
        }
    }
}
