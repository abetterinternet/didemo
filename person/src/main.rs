use axum::{Json, Router, extract::State, routing::get};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    router::actor_main,
};
use didemo_person::PresentedCredential;
use serde::{Deserialize, Serialize};

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// The person's name.
    name: String,

    /// The person's age in years.
    age: u32,
}

impl Configuration for PersonConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config| {
        let routes = Router::new()
            .route("/config", get(serve_config))
            .route("/present", get(present))
            .route("/wallet-config", get(wallet_config))
            .with_state(config);

        Ok(("person", routes))
    })
    .await?;

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

async fn wallet_config() -> String {
    tracing::info!("serving wallet config");
    reqwest::get("http://issuer/config")
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
