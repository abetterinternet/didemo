use anyhow::{Context, anyhow};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use didemo_common::{
    bbs::BbsKeypair,
    config::{CommonConfiguration, Configuration},
    credential::{
        Credential, CredentialSignature, CredentialType, DriversLicense, DriversLicenseRequest,
        LibraryCard, LibraryCardRequest,
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
    actor_name: String,
    http_client: Client,
    last_serial_number: u64,
    bbs_keypair: BbsKeypair,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config: IssuerConfiguration, client_builder| {
        let http_client = client_builder.build()?;

        let actor_name = format!("issuer/{}", config.label);

        // Using this fixed seed is not secure but this is harmless in the simulation setup.
        let bbs_keypair = BbsKeypair::new(&actor_name)?;

        let issuer = Issuer {
            config,
            actor_name: actor_name.clone(),
            http_client,
            last_serial_number: 0,
            bbs_keypair,
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
    let mut issuer = issuer.lock().await;

    // This is where an issuer would perform actual verification that the person is a legitimate
    // member of some group, citizen of some country, allowed to operate a motor vehicle or
    // whatever. We do not simulate such verification and assume they succeed.

    // TODO: issuer should verify that it trusts the wallet (i.e. that the wallet was made by an
    // authorized vendor and has needed capabilities).

    // TODO: other policy checks? Uniqueness of certain fields?

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

    let (bbs_messages, encoded_credential) = match request.credential_type {
        CredentialType::LibraryCard => {
            let library_card_request: LibraryCardRequest =
                serde_json::from_str(&request.requested_credential)
                    .context("failed to deserialize library card request")?;

            let messages = Vec::from([
                issuer.config.label.clone().into_bytes(),
                library_card_request.holder_name.clone().into_bytes(),
                issuer.last_serial_number.to_be_bytes().to_vec(),
            ]);
            let issued_credential = serde_json::to_string(&LibraryCard {
                library_name: issuer.config.label.clone(),
                holder_name: library_card_request.holder_name,
                serial_number: issuer.last_serial_number,
            })
            .context("failed to serialize library card")?;

            (messages, issued_credential)
        }
        CredentialType::DriversLicense => {
            let drivers_license_request: DriversLicenseRequest =
                serde_json::from_str(&request.requested_credential)
                    .context("failed to deserialize driver's license request")?;

            let messages = Vec::from([
                issuer.config.label.clone().into_bytes(),
                drivers_license_request.holder_name.clone().into_bytes(),
                issuer.last_serial_number.to_be_bytes().to_vec(),
                drivers_license_request.home_address.clone().into_bytes(),
                if drivers_license_request.organ_donor {
                    Vec::from([1])
                } else {
                    Vec::from([0])
                },
                drivers_license_request.birthdate.to_be_bytes().to_vec(),
            ]);
            let issued_credential = serde_json::to_string(&DriversLicense {
                issuing_jurisdiction: issuer.config.label.clone(),
                holder_name: drivers_license_request.holder_name,
                serial_number: issuer.last_serial_number,
                home_address: drivers_license_request.home_address,
                organ_donor: drivers_license_request.organ_donor,
                birthdate: drivers_license_request.birthdate,
            })
            .context("failed to serialize driver's license")?;

            (messages, issued_credential)
        }
    };

    let header = issuer.actor_name.as_bytes().to_vec();
    let signature = issuer
        .bbs_keypair
        .sign(header.clone(), bbs_messages.clone())?;

    let issued_credential = Credential {
        credential_type: request.credential_type,
        encoded_credential,
        signature: CredentialSignature { signature, header },
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

    tracing::info!(
        credential_type = ?request.credential_type,
        wallet_hostname = request.wallet_hostname,
        "issued credential"
    );

    Ok(StatusCode::CREATED)
}

/// Print the configuration.
async fn serve_config(State(issuer): State<Arc<Mutex<Issuer>>>) -> Json<IssuerConfiguration> {
    tracing::info!("serving config endpoint");
    Json(issuer.lock().await.config.clone())
}
