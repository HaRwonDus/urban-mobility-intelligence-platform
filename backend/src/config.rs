use anyhow::Context;
use std::{env, net::SocketAddr};

#[derive(Clone)]
pub struct Config {
    pub api_addr: SocketAddr,
    pub database_url: String,
    pub dgis_api_key: String,
    pub iqair_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_addr = env::var("API_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8000".to_string())
            .parse()
            .context("API_ADDR must be a valid socket address")?;

        Ok(Self {
            api_addr,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            dgis_api_key: env::var("DGIS_API_KEY").unwrap_or_else(|_| "replace_me".to_string()),
            iqair_api_key: env::var("IQAIR_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty() && value != "replace_me"),
        })
    }
}
