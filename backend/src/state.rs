use crate::{
    config::Config,
    services::{dgis::DgisClient, iqair::IqAirClient},
};
use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub dgis: DgisClient,
    pub iqair: IqAirClient,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(8)
            .connect(&config.database_url)
            .await
            .context("failed to connect to PostgreSQL")?;

        let dgis = DgisClient::new(config.dgis_api_key.clone());
        let iqair = IqAirClient::new(config.iqair_api_key.clone());

        Ok(Self { db, dgis, iqair })
    }
}
