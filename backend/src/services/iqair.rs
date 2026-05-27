use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone)]
pub struct IqAirClient {
    api_key: Option<String>,
    http: Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct IqAirReading {
    pub aqi_us: i32,
    pub main_pollutant: String,
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<i32>,
    pub source: String,
}

impl IqAirClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            http: Client::new(),
        }
    }

    pub async fn nearest_city(&self, lat: f64, lon: f64) -> Result<Option<IqAirReading>> {
        let Some(api_key) = self.api_key.as_deref() else {
            return Ok(None);
        };

        let body: Value = self
            .http
            .get("https://api.airvisual.com/v2/nearest_city")
            .query(&[
                ("lat", lat.to_string()),
                ("lon", lon.to_string()),
                ("key", api_key.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let pollution = body.pointer("/data/current/pollution");
        let weather = body.pointer("/data/current/weather");
        let Some(aqi_us) = pollution
            .and_then(|value| value.get("aqius"))
            .and_then(Value::as_i64)
        else {
            return Ok(None);
        };

        Ok(Some(IqAirReading {
            aqi_us: aqi_us as i32,
            main_pollutant: pollution
                .and_then(|value| value.get("mainus"))
                .and_then(Value::as_str)
                .unwrap_or("pm25")
                .to_string(),
            temperature_c: weather
                .and_then(|value| value.get("tp"))
                .and_then(Value::as_f64),
            humidity_pct: weather
                .and_then(|value| value.get("hu"))
                .and_then(Value::as_i64)
                .map(|value| value as i32),
            source: "IQAir AirVisual API".to_string(),
        }))
    }
}
