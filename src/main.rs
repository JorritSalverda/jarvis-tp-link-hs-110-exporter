mod bigquery_client;
mod config_client;
mod exporter_service;
mod hs110_client;
mod model;
mod state_client;

use bigquery_client::{BigqueryClient, BigqueryClientConfig};
use config_client::{ConfigClient, ConfigClientConfig};
use exporter_service::{ExporterService, ExporterServiceConfig};
use hs110_client::{HS110Client, HS110ClientConfig};
use state_client::{StateClient, StateClientConfig};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hs110_client_config = HS110ClientConfig::from_env()?;
    let hs110_client = HS110Client::new(hs110_client_config);

    let state_client_config = StateClientConfig::from_env().await?;
    let state_client = StateClient::new(state_client_config);

    let bigquery_client_config = BigqueryClientConfig::from_env().await?;
    let bigquery_client = BigqueryClient::new(bigquery_client_config);

    let config_client_config = ConfigClientConfig::from_env()?;
    let config_client = ConfigClient::new(config_client_config);

    let exporter_service_config =
        ExporterServiceConfig::new(config_client, bigquery_client, state_client, hs110_client)?;
    let exporter_service = ExporterService::new(exporter_service_config);

    exporter_service.run().await?;

    Ok(())
}
