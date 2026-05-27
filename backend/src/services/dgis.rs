use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::env;

#[derive(Clone)]
pub struct DgisClient {
    api_key: String,
    http: Client,
}

impl DgisClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            api_key: env::var("DGIS_API_KEY").context("DGIS_API_KEY is required")?,
            http: Client::new(),
        })
    }

    pub async fn search_places(
        &self,
        query: &str,
        city: Option<&str>,
        object_type: Option<&str>,
    ) -> Result<Vec<Value>> {
        let mut request = self
            .http
            .get("https://catalog.api.2gis.com/3.0/items")
            .query(&[("q", query), ("key", self.api_key.as_str())]);

        if let Some(city) = city {
            request = request.query(&[("city_name", city)]);
        }

        if let Some(object_type) = object_type {
            request = request.query(&[("type", object_type)]);
        }

        let body: Value = request.send().await?.error_for_status()?.json().await?;
        let items = body
            .pointer("/result/items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        Ok(items)
    }
}
