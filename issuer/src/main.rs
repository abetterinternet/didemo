use anyhow::{Context, anyhow};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    credential::{
        Credential, CredentialType, DriversLicense, DriversLicenseRequest, LibraryCard,
        LibraryCardRequest,
    },
    messages::issuer::IssueCredentialRequest,
    router::{AppError, actor_main},
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssuerConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// A label identifying the issuer.
    label: String,

    /// Credentials this issuer is allowed to issue
    credential_types: Vec<CredentialType>,
}

impl Configuration for IssuerConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

struct Issuer {
    config: IssuerConfiguration,
    http_client: Client,
    last_serial_number: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config: IssuerConfiguration, client_builder| {
        let http_client = client_builder.build()?;

        let actor_name = format!("issuer/{}", config.label);
        let issuer = Issuer {
            config,
            http_client,
            last_serial_number: 0,
        };

        let routes = Router::new()
            .route("/config", get(serve_config))
            .route("/issue", put(issue_credential))
            .with_state(Arc::new(Mutex::new(issuer)));

        Ok((actor_name, routes))
    })
    .await?;

    Ok(())
}

/// Issue the requested credential.
#[axum::debug_handler]
async fn issue_credential(
    State(issuer): State<Arc<Mutex<Issuer>>>,
    Json(request): Json<IssueCredentialRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("issuing a credential");
    let mut issuer = issuer.lock().await;

    // This is where an issuer would perform actual verification that the person is a legitimate
    // member of some group, citizen of some country, allowed to operate a motor vehicle or
    // whatever. We do not simulate such verification and assume they succeed.

    // TODO: issuer should verify that it trusts the wallet (i.e. that the wallet was made by an
    // authorized vendor and has needed capabilities).

    // TODO: other policy checks? Uniqueness of certain fields?

    // TODO: actual cryptographic signing!
    if !issuer
        .config
        .credential_types
        .contains(&request.credential_type)
    {
        return Err(anyhow!(
            "not permitted to issue credential {:?}",
            request.credential_type
        )
        .into());
    }

    issuer.last_serial_number += 1;

    let issued_credential = Credential {
        credential_type: request.credential_type,
        encoded_credential: match request.credential_type {
            CredentialType::LibraryCard => {
                let library_card_request: LibraryCardRequest =
                    serde_json::from_str(&request.requested_credential)
                        .context("failed to deserialize library card request")?;
                serde_json::to_string(&LibraryCard {
                    library_name: issuer.config.label.clone(),
                    holder_name: library_card_request.holder_name,
                    serial_number: issuer.last_serial_number,
                })
                .context("failed to serialize library card")?
            }
            CredentialType::DriversLicense => {
                let drivers_license_request: DriversLicenseRequest =
                    serde_json::from_str(&request.requested_credential)
                        .context("failed to deserialize driver's license request")?;
                serde_json::to_string(&DriversLicense {
                    issuing_jurisdiction: issuer.config.label.clone(),
                    holder_name: drivers_license_request.holder_name,
                    serial_number: issuer.last_serial_number,
                    home_address: drivers_license_request.home_address,
                    organ_donor: drivers_license_request.organ_donor,
                    birthdate: drivers_license_request.birthdate,
                })
                .context("failed to serialize driver's license")?
            }
        },
    };

    let wallet_response = issuer
        .http_client
        .put(format!("http://{}/credentials", request.wallet_hostname))
        .json(&issued_credential)
        .send()
        .await
        .context("failed to send request to issuer")?;

    if !wallet_response.status().is_success() {
        // TODO: augment handlers so we can send a non-200 response with a descriptive body

        return Err(anyhow!(
            "request to wallet failed: {:?}",
            wallet_response.error_for_status()
        )
        .into());
    }

    Ok(StatusCode::CREATED)
}

/// Print the configuration.
async fn serve_config(State(issuer): State<Arc<Mutex<Issuer>>>) -> Json<IssuerConfiguration> {
    tracing::info!("serving config endpoint");
    Json(issuer.lock().await.config.clone())
}
