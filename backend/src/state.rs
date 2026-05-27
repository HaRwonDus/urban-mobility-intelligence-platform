use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn new(database_url: String) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(8)
            .connect(&database_url)
            .await
            .context("failed to connect to PostgreSQL")?;

        Ok(Self { db })
    }
}
