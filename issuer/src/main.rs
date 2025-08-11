use axum::{Json, Router, extract::State, routing::get};
use didemo_common::{
    config::{CommonConfiguration, Configuration},
    router::actor_main,
};
use serde::{Deserialize, Serialize};

/// Configuration for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssuerConfiguration {
    #[serde(flatten)]
    common: CommonConfiguration,

    /// A label identifying the issuer.
    label: String,
}

impl Configuration for IssuerConfiguration {
    fn common_configuration(&self) -> &CommonConfiguration {
        &self.common
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    actor_main(|config| {
        let routes = Router::new()
            .route("/config", get(serve_config))
            .with_state(config);

        Ok(("issuer", routes))
    })
    .await?;

    Ok(())
}

/// Print the configuration.
async fn serve_config(State(config): State<IssuerConfiguration>) -> Json<IssuerConfiguration> {
    tracing::info!("serving config endpoint");
    Json(config)
}
