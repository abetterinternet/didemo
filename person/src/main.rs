use anyhow::{Context, anyhow};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    credential::{CredentialType, DriversLicenseRequest, LibraryCardRequest},
    messages::{
        issuer::IssueCredentialRequest,
        person::{ObtainCredentialRequest, PresentedCredential},
    },
    router::{AppError, actor_main},
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// The person's name.
    name: String,

    /// The person's home address (opaque string).
    // TODO: needs a richer representation if we want to do something like a zero knowledge proof
    // of residency in some jursidiction.
    home_address: String,

    /// Whether the person is an organ donor.
    organ_donor: bool,

    /// The person's birthdate, as seconds since the Unix epoch.
    birthdate: u64,

    /// The hostname at which this person's wallet can be reached.
    // TODO: this should be dynamically settable using some kind of route simulating a wallet
    // purchase.
    wallet_hostname: String,
}

impl Configuration for PersonConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config: PersonConfiguration, client_builder| {
        let client = client_builder.build()?;
        let actor_name = format!("person/{}", config.name);

        let routes = Router::new()
            .route("/credential", post(obtain_credential))
            .route("/present", get(present))
            .with_state((config, client));

        Ok((actor_name, routes))
    })
    .await?;

    Ok(())
}

/// Instruct the person to obtain a credential from the designated issuer
#[axum::debug_handler]
async fn obtain_credential(
    State((config, http_client)): State<(PersonConfiguration, Client)>,
    Json(request): Json<ObtainCredentialRequest>,
) -> Result<StatusCode, AppError> {
    let issue_request = IssueCredentialRequest {
        credential_type: request.credential_type,
        requested_credential: match request.credential_type {
            CredentialType::LibraryCard => serde_json::to_string(&LibraryCardRequest {
                holder_name: config.name,
            })
            .context("failed to serialize credential")?,
            CredentialType::DriversLicense => serde_json::to_string(&DriversLicenseRequest {
                holder_name: config.name,
                home_address: config.home_address,
                organ_donor: config.organ_donor,
                birthdate: config.birthdate,
            })
            .context("failed to serialize credential")?,
        },
        wallet_hostname: config.wallet_hostname,
    };

    let issue_response = http_client
        .put(format!("http://{}/issue", request.issuer))
        .json(&issue_request)
        .send()
        .await
        .context("failed to send request to issuer")?;

    if !issue_response.status().is_success() {
        // TODO: augment handlers so we can send a non-200 response with a descriptive body

        return Err(anyhow!(
            "request to issuer failed: {:?}",
            issue_response.error_for_status()
        )
        .into());
    }

    Ok(StatusCode::CREATED)
}

/// Present this person's credentials.
#[axum::debug_handler]
async fn present(
    State((config, _http_client)): State<(PersonConfiguration, Client)>,
) -> Json<PresentedCredential> {
    tracing::info!("presenting credential");
    Json(PresentedCredential {
        name: config.name,
        birthdate: config.birthdate,
    })
}
