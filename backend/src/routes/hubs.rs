use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/hubs", get(hubs_analytics))
        .route("/api/hubs/proposals", post(analyze_hub_proposal))
        .route("/api/coverage/gaps", get(coverage_gaps))
}

#[derive(Debug, Serialize, FromRow, Clone)]
struct HubRow {
    id: Uuid,
    name: String,
    hub_type: String,
    lat: f64,
    lon: f64,
    address: Option<String>,
    avg_daily_arrivals: i32,
    avg_daily_departures: i32,
    nearest_district: Option<String>,
    nearest_district_score: Option<f64>,
    access_time_min: f64,
}

#[derive(Debug, Serialize)]
struct HubAnalytics {
    id: Uuid,
    name: String,
    hub_type: String,
    lat: f64,
    lon: f64,
    address: Option<String>,
    avg_daily_arrivals: i32,
    avg_daily_departures: i32,
    avg_daily_flow: i32,
    nearest_district: Option<String>,
    access_time_min: i32,
    pressure_index: i32,
    recommendation: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct HubProposalRequest {
    name: Option<String>,
    hub_type: String,
    lat: f64,
    lon: f64,
    daily_capacity: Option<i32>,
    greenfield: Option<bool>,
}

#[derive(Debug, Serialize)]
struct HubProposalAnalysis {
    name: String,
    hub_type: String,
    lat: f64,
    lon: f64,
    nearest_district: String,
    underserved_score: i32,
    network_fit_score: i32,
    duplicate_pressure: i32,
    estimated_daily_flow: i32,
    verdict: String,
    signals: Vec<String>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
struct DistrictCoverageRow {
    district: String,
    lat: f64,
    lon: f64,
    population: Option<i32>,
    avg_time_to_stop_min: f64,
    avg_time_to_hub_min: f64,
    score: f64,
}

#[derive(Debug, Serialize)]
struct CoverageGap {
    id: String,
    district: String,
    lat: f64,
    lon: f64,
    severity: String,
    isolation_score: i32,
    reason: String,
}

async fn hubs_analytics(State(state): State<AppState>) -> Result<Json<Vec<HubAnalytics>>, String> {
    let rows = load_hubs(&state).await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let flow = row.avg_daily_arrivals + row.avg_daily_departures;
                let district_score = row.nearest_district_score.unwrap_or(50.0);
                let pressure = ((flow as f64 / 420.0) + row.access_time_min * 1.6
                    + (65.0 - district_score).max(0.0) * 0.45)
                    .round()
                    .clamp(0.0, 100.0) as i32;
                HubAnalytics {
                    id: row.id,
                    name: row.name,
                    hub_type: row.hub_type.clone(),
                    lat: row.lat,
                    lon: row.lon,
                    address: row.address,
                    avg_daily_arrivals: row.avg_daily_arrivals,
                    avg_daily_departures: row.avg_daily_departures,
                    avg_daily_flow: flow,
                    nearest_district: row.nearest_district,
                    access_time_min: row.access_time_min.round() as i32,
                    pressure_index: pressure,
                    recommendation: hub_recommendation(&row.hub_type, pressure).to_string(),
                    source: "seed/planning estimates; ready for airport, rail and ticketing feeds"
                        .to_string(),
                }
            })
            .collect(),
    ))
}

async fn analyze_hub_proposal(
    State(state): State<AppState>,
    Json(payload): Json<HubProposalRequest>,
) -> Result<Json<HubProposalAnalysis>, String> {
    let districts = load_coverage_rows(&state).await?;
    let hubs = load_hubs(&state).await?;
    let Some(nearest) = districts.iter().min_by(|a, b| {
        haversine_km(payload.lat, payload.lon, a.lat, a.lon)
            .total_cmp(&haversine_km(payload.lat, payload.lon, b.lat, b.lon))
    }) else {
        return Err("No districts available".to_string());
    };

    let nearest_hub_km = hubs
        .iter()
        .filter(|hub| hub.hub_type == payload.hub_type)
        .map(|hub| haversine_km(payload.lat, payload.lon, hub.lat, hub.lon))
        .fold(f64::INFINITY, f64::min);
    let capacity = payload.daily_capacity.unwrap_or_else(|| {
        if payload.hub_type == "station" {
            14000
        } else {
            9000
        }
    });
    let greenfield = payload.greenfield.unwrap_or(false);
    let underserved_score = ((100.0 - nearest.score) * 0.62
        + nearest.avg_time_to_hub_min * 1.25
        + nearest.avg_time_to_stop_min * 0.85)
        .round()
        .clamp(0.0, 100.0) as i32;
    let duplicate_pressure = if greenfield {
        0
    } else {
        ((18.0 - nearest_hub_km).max(0.0) * 5.5).round().clamp(0.0, 100.0) as i32
    };
    let network_fit_score =
        (underserved_score as f64 * 0.58 + (100 - duplicate_pressure) as f64 * 0.32
            + (capacity as f64 / 650.0).min(20.0))
            .round()
            .clamp(0.0, 100.0) as i32;
    let estimated_daily_flow = ((nearest.population.unwrap_or(180000) as f64 * 0.035)
        + capacity as f64 * 0.42
        + underserved_score as f64 * 55.0)
        .round() as i32;

    Ok(Json(HubProposalAnalysis {
        name: payload
            .name
            .unwrap_or_else(|| format!("Proposed {}", payload.hub_type)),
        hub_type: payload.hub_type,
        lat: payload.lat,
        lon: payload.lon,
        nearest_district: nearest.district.clone(),
        underserved_score,
        network_fit_score,
        duplicate_pressure,
        estimated_daily_flow,
        verdict: if network_fit_score >= 70 {
            "Strong candidate for a new intermodal hub scenario.".to_string()
        } else if network_fit_score >= 45 {
            "Promising, but test feeder routes and access roads before approval.".to_string()
        } else {
            "Weak location unless this is a greenfield network built from zero.".to_string()
        },
        signals: vec![
            format!("Nearest district: {}", nearest.district),
            format!("District accessibility score: {:.0}", nearest.score),
            format!("Time to existing hub: {:.0} min", nearest.avg_time_to_hub_min),
            format!(
                "Nearest same-type hub: {}",
                if nearest_hub_km.is_finite() {
                    format!("{nearest_hub_km:.1} km")
                } else {
                    "none".to_string()
                }
            ),
        ],
    }))
}

async fn coverage_gaps(State(state): State<AppState>) -> Result<Json<Vec<CoverageGap>>, String> {
    let rows = load_coverage_rows(&state).await?;
    Ok(Json(
        rows.into_iter()
            .filter_map(|row| {
                let isolation = (row.avg_time_to_stop_min * 2.1
                    + row.avg_time_to_hub_min * 1.55
                    + (55.0 - row.score).max(0.0) * 0.8)
                    .round()
                    .clamp(0.0, 100.0) as i32;
                if isolation < 52 {
                    return None;
                }

                Some(CoverageGap {
                    id: slug(&row.district),
                    district: row.district.clone(),
                    lat: row.lat,
                    lon: row.lon,
                    severity: if isolation >= 72 { "high" } else { "medium" }.to_string(),
                    isolation_score: isolation,
                    reason: format!(
                        "{:.0} min to stop, {:.0} min to intermodal hub",
                        row.avg_time_to_stop_min, row.avg_time_to_hub_min
                    ),
                })
            })
            .collect(),
    ))
}

async fn load_hubs(state: &AppState) -> Result<Vec<HubRow>, String> {
    sqlx::query_as::<_, HubRow>(
        r#"
        SELECT
          o.id,
          o.name,
          o.type::text AS hub_type,
          o.lat,
          o.lon,
          o.address,
          COALESCE((o.raw ->> 'avg_daily_arrivals')::int, 0) AS avg_daily_arrivals,
          COALESCE((o.raw ->> 'avg_daily_departures')::int, 0) AS avg_daily_departures,
          nearest_district.name AS nearest_district,
          nearest_score.score::float8 AS nearest_district_score,
          COALESCE(ST_Distance(o.geom, nearest_stop.geom) / 80.0, 30.0)::float8 AS access_time_min
        FROM city_objects o
        LEFT JOIN LATERAL (
          SELECT d.id, d.name
          FROM districts d
          ORDER BY o.geom <-> d.geom
          LIMIT 1
        ) nearest_district ON true
        LEFT JOIN LATERAL (
          SELECT ms.score
          FROM mobility_scores ms
          WHERE ms.district_id = nearest_district.id
          ORDER BY ms.calculated_at DESC
          LIMIT 1
        ) nearest_score ON true
        LEFT JOIN LATERAL (
          SELECT s.geom
          FROM city_objects s
          WHERE s.type::text IN ('stop', 'metro', 'station', 'hub', 'bus_station')
            AND s.id <> o.id
          ORDER BY ST_Distance(o.geom, s.geom)
          LIMIT 1
        ) nearest_stop ON true
        WHERE o.type::text IN ('airport', 'station', 'bus_station')
        ORDER BY avg_daily_arrivals + avg_daily_departures DESC, o.name
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())
}

async fn load_coverage_rows(state: &AppState) -> Result<Vec<DistrictCoverageRow>, String> {
    sqlx::query_as::<_, DistrictCoverageRow>(
        r#"
        SELECT
          d.name AS district,
          d.lat,
          d.lon,
          d.population,
          COALESCE(ms.avg_time_to_stop_min, 24)::float8 AS avg_time_to_stop_min,
          COALESCE(ms.avg_time_to_hub_min, 30)::float8 AS avg_time_to_hub_min,
          COALESCE(ms.score, 45)::float8 AS score
        FROM districts d
        LEFT JOIN LATERAL (
          SELECT avg_time_to_stop_min, avg_time_to_hub_min, score
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
    .map_err(|err| err.to_string())
}

fn hub_recommendation(hub_type: &str, pressure: i32) -> &'static str {
    if pressure >= 70 && hub_type == "airport" {
        "Add express airport transit and protect peak-hour bus priority."
    } else if pressure >= 70 {
        "Add feeder routes, park-and-ride capacity and dedicated transfer stops."
    } else if pressure >= 45 {
        "Monitor peak flows and improve first/last-mile coverage."
    } else {
        "Hub is stable under current planning assumptions."
    }
}

fn haversine_km(origin_lat: f64, origin_lon: f64, destination_lat: f64, destination_lon: f64) -> f64 {
    let radius_km = 6371.0;
    let d_lat = (destination_lat - origin_lat).to_radians();
    let d_lon = (destination_lon - origin_lon).to_radians();
    let lat1 = origin_lat.to_radians();
    let lat2 = destination_lat.to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * radius_km * a.sqrt().asin()
}

fn slug(value: &str) -> String {
    value.to_ascii_lowercase().replace(' ', "-")
}
