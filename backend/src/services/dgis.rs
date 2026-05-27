use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct DgisClient {
    api_key: String,
    http: Client,
}

impl DgisClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: Client::new(),
        }
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
            .query(&[
                ("q", query),
                ("key", self.api_key.as_str()),
                ("location", "76.9286,43.2489"),
                ("radius", "30000"),
                ("fields", "items.point,items.address,items.full_name"),
            ]);

        if let Some(city) = city.filter(|value| !value.eq_ignore_ascii_case("almaty")) {
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
