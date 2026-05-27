use crate::{services::iqair::IqAirReading, state::AppState};
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/air-quality", get(list_air_quality))
}

#[derive(Debug, FromRow)]
struct DistrictAirBase {
    name: String,
    lat: f64,
    lon: f64,
    population: Option<i32>,
    score: Option<f64>,
}

#[derive(Debug, Serialize)]
struct AirQualityPoint {
    district: String,
    lat: f64,
    lon: f64,
    aqi_us: i32,
    category: String,
    main_pollutant: String,
    temperature_c: Option<f64>,
    humidity_pct: Option<i32>,
    health_note: String,
    source: String,
}

async fn list_air_quality(State(state): State<AppState>) -> Result<Json<Vec<AirQualityPoint>>, String> {
    let districts = sqlx::query_as::<_, DistrictAirBase>(
        r#"
        SELECT
          d.name,
          d.lat,
          d.lon,
          d.population,
          ms.score::float8 AS score
        FROM districts d
        LEFT JOIN LATERAL (
          SELECT score
          FROM mobility_scores
          WHERE district_id = d.id
          ORDER BY calculated_at DESC
          LIMIT 1
        ) ms ON true
        ORDER BY d.name
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    let mut points = Vec::with_capacity(districts.len());
    for district in districts {
        let reading = state
            .iqair
            .nearest_city(district.lat, district.lon)
            .await
            .ok()
            .flatten();
        points.push(to_air_quality_point(district, reading));
    }

    Ok(Json(points))
}

fn to_air_quality_point(base: DistrictAirBase, reading: Option<IqAirReading>) -> AirQualityPoint {
    let fallback_aqi = estimate_demo_aqi(base.population, base.score);
    let aqi = reading.as_ref().map(|item| item.aqi_us).unwrap_or(fallback_aqi);
    let source = reading
        .as_ref()
        .map(|item| item.source.clone())
        .unwrap_or_else(|| "MVP estimated layer; add IQAIR_API_KEY for live IQAir data".to_string());

    AirQualityPoint {
        district: base.name,
        lat: base.lat,
        lon: base.lon,
        aqi_us: aqi,
        category: category_for(aqi).to_string(),
        main_pollutant: reading
            .as_ref()
            .map(|item| item.main_pollutant.clone())
            .unwrap_or_else(|| "PM2.5".to_string()),
        temperature_c: reading.as_ref().and_then(|item| item.temperature_c),
        humidity_pct: reading.as_ref().and_then(|item| item.humidity_pct),
        health_note: health_note_for(aqi).to_string(),
        source,
    }
}

fn estimate_demo_aqi(population: Option<i32>, score: Option<f64>) -> i32 {
    let population_pressure = population.unwrap_or(220_000) as f64 / 12_000.0;
    let mobility_pressure = (70.0 - score.unwrap_or(52.0)).max(0.0) * 0.55;
    (42.0 + population_pressure + mobility_pressure).round().clamp(35.0, 118.0) as i32
}

fn category_for(aqi: i32) -> &'static str {
    match aqi {
        0..=50 => "Good",
        51..=100 => "Moderate",
        101..=150 => "Unhealthy for sensitive groups",
        151..=200 => "Unhealthy",
        201..=300 => "Very unhealthy",
        _ => "Hazardous",
    }
}

fn health_note_for(aqi: i32) -> &'static str {
    match aqi {
        0..=50 => "clean baseline",
        51..=100 => "watch school and hospital corridors",
        101..=150 => "prioritize low-emission transit links",
        151..=200 => "reduce exposure near overloaded corridors",
        _ => "emergency exposure mitigation needed",
    }
}
