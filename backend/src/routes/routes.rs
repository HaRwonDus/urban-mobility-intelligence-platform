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
        .route("/routes", post(create_route))
        .route("/api/routes", get(routes_overview))
        .route("/api/routes/gaps", get(route_gaps))
        .route("/api/routes/suggestions", get(route_suggestions))
        .route("/api/routes/matrix", get(route_matrix))
        .route("/api/routes/simulate", post(simulate_route))
}

#[derive(Debug, Deserialize)]
struct CreateRouteRequest {
    origin_id: Option<Uuid>,
    destination_id: Option<Uuid>,
    distance_m: i32,
    duration_sec: i32,
    transport_type: String,
}

#[derive(Debug, Serialize)]
struct CreateRouteResponse {
    id: Uuid,
}

#[derive(Debug, Serialize, FromRow, Clone)]
struct DistrictMetric {
    name: String,
    lat: f64,
    lon: f64,
    population: Option<i32>,
    avg_time_to_hub_min: f64,
    poi_density: f64,
    connectivity_score: f64,
    score: f64,
}

#[derive(Debug, Serialize, Clone)]
struct RouteGap {
    id: String,
    origin: String,
    destination: String,
    current_time_min: i32,
    target_time_min: i32,
    problem: String,
    suggestion: String,
    priority: String,
}

#[derive(Debug, Serialize, Clone)]
struct SuggestedRoute {
    id: String,
    name: String,
    origin: String,
    via: Vec<String>,
    destination: String,
    route_type: String,
    expected_impact: i32,
    confidence: f64,
}

#[derive(Debug, Serialize, Clone)]
struct DuplicatedCoverage {
    corridor: String,
    problem: String,
    action: String,
    severity: String,
}

#[derive(Debug, Serialize, Clone)]
struct RouteComparison {
    district: String,
    current_score: i32,
    projected_score: i32,
    time_saved_min: i32,
    affected_poi: i32,
    priority: String,
}

#[derive(Debug, Serialize, Clone)]
struct RouteMatrixCell {
    origin: String,
    destination: String,
    current_time_min: i32,
    target_time_min: i32,
    gap_min: i32,
    priority: String,
}

#[derive(Debug, Serialize)]
struct RoutesOverview {
    gaps: Vec<RouteGap>,
    suggestions: Vec<SuggestedRoute>,
    duplicated_coverage: Vec<DuplicatedCoverage>,
    comparison: Vec<RouteComparison>,
}

#[derive(Debug, Deserialize)]
struct SimulateRouteRequest {
    origin: Option<String>,
    destination: Option<String>,
    intervention: Option<String>,
}

#[derive(Debug, Serialize)]
struct SimulateRouteResponse {
    origin: String,
    destination: String,
    current_time_min: i32,
    suggested_route: String,
    projected_time_min: i32,
    time_saved_min: i32,
    projected_score_gain: i32,
    confidence: f64,
}

async fn create_route(
    State(state): State<AppState>,
    Json(payload): Json<CreateRouteRequest>,
) -> Result<Json<CreateRouteResponse>, String> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO routes (origin_id, destination_id, distance_m, duration_sec, transport_type)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.origin_id)
    .bind(payload.destination_id)
    .bind(payload.distance_m)
    .bind(payload.duration_sec)
    .bind(payload.transport_type)
    .fetch_one(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(CreateRouteResponse { id }))
}

async fn routes_overview(State(state): State<AppState>) -> Result<Json<RoutesOverview>, String> {
    let metrics = load_district_metrics(&state).await?;
    Ok(Json(RoutesOverview {
        gaps: build_gaps(&metrics),
        suggestions: build_suggestions(&metrics),
        duplicated_coverage: build_duplicated_coverage(&metrics),
        comparison: build_comparison(&metrics),
    }))
}

async fn route_gaps(State(state): State<AppState>) -> Result<Json<Vec<RouteGap>>, String> {
    let metrics = load_district_metrics(&state).await?;
    Ok(Json(build_gaps(&metrics)))
}

async fn route_suggestions(State(state): State<AppState>) -> Result<Json<Vec<SuggestedRoute>>, String> {
    let metrics = load_district_metrics(&state).await?;
    Ok(Json(build_suggestions(&metrics)))
}

async fn route_matrix(State(state): State<AppState>) -> Result<Json<Vec<RouteMatrixCell>>, String> {
    let metrics = load_district_metrics(&state).await?;
    Ok(Json(build_matrix(&metrics)))
}

async fn simulate_route(
    State(state): State<AppState>,
    Json(payload): Json<SimulateRouteRequest>,
) -> Result<Json<SimulateRouteResponse>, String> {
    let metrics = load_district_metrics(&state).await?;
    let origin = payload.origin.unwrap_or_else(|| "Nauryzbay".to_string());
    let destination = payload
        .destination
        .unwrap_or_else(|| "Almalinsky".to_string());
    let intervention = payload
        .intervention
        .unwrap_or_else(|| "express feeder".to_string());

    let origin_metric = find_metric(&metrics, &origin)
        .or_else(|| metrics.first())
        .ok_or_else(|| "No districts available".to_string())?;
    let destination_metric = find_metric(&metrics, &destination).unwrap_or(origin_metric);
    let current_time = estimate_connection_time(origin_metric, destination_metric);
    let target_time = estimate_target_time(origin_metric, destination_metric);
    let projected_time = (current_time - improvement_for(&intervention)).max(target_time + 3);
    let time_saved = (current_time - projected_time).max(0);

    Ok(Json(SimulateRouteResponse {
        origin: origin_metric.name.clone(),
        destination: destination_metric.name.clone(),
        current_time_min: current_time,
        suggested_route: intervention,
        projected_time_min: projected_time,
        time_saved_min: time_saved,
        projected_score_gain: (time_saved as f64 * 0.72).round() as i32,
        confidence: 0.78,
    }))
}

async fn load_district_metrics(state: &AppState) -> Result<Vec<DistrictMetric>, String> {
    sqlx::query_as::<_, DistrictMetric>(
        r#"
        SELECT
          d.name,
          d.lat,
          d.lon,
          d.population,
          COALESCE(ms.avg_time_to_hub_min, 24)::float8 AS avg_time_to_hub_min,
          COALESCE(ms.poi_density, 0)::float8 AS poi_density,
          COALESCE(ms.connectivity_score, 45)::float8 AS connectivity_score,
          COALESCE(ms.score, 50)::float8 AS score
        FROM districts d
        LEFT JOIN LATERAL (
          SELECT avg_time_to_hub_min, poi_density, connectivity_score, score
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

fn build_gaps(metrics: &[DistrictMetric]) -> Vec<RouteGap> {
    let mut gaps: Vec<RouteGap> = metrics
        .iter()
        .flat_map(|origin| {
            metrics.iter().filter_map(move |destination| {
                if origin.name == destination.name {
                    return None;
                }

                let current = estimate_connection_time(origin, destination);
                let target = estimate_target_time(origin, destination);
                let weak_score = origin.score.min(destination.score);
                if current < target + 14 && weak_score >= 45.0 {
                    return None;
                }

                let priority = if current - target >= 24 || weak_score < 35.0 {
                    "high"
                } else {
                    "medium"
                };

                Some(RouteGap {
                    id: format!("{}-{}", slug(&origin.name), slug(&destination.name)),
                    origin: origin.name.clone(),
                    destination: destination.name.clone(),
                    current_time_min: current,
                    target_time_min: target,
                    problem: "weak trunk connection".to_string(),
                    suggestion: "express feeder route to metro/BRT hub".to_string(),
                    priority: priority.to_string(),
                })
            })
        })
        .collect();

    gaps.sort_by(|a, b| {
        let a_gap = a.current_time_min - a.target_time_min;
        let b_gap = b.current_time_min - b.target_time_min;
        b_gap.cmp(&a_gap)
    });
    gaps.truncate(6);
    gaps
}

fn build_suggestions(metrics: &[DistrictMetric]) -> Vec<SuggestedRoute> {
    build_gaps(metrics)
        .into_iter()
        .take(4)
        .enumerate()
        .map(|(index, gap)| SuggestedRoute {
            id: format!("R-{:02}", index + 1),
            name: format!("{} to {} express", gap.origin, gap.destination),
            origin: gap.origin,
            via: suggested_via(index),
            destination: gap.destination,
            route_type: if index == 0 {
                "express bus / BRT feeder".to_string()
            } else {
                "express bus".to_string()
            },
            expected_impact: (gap.current_time_min - gap.target_time_min).max(10),
            confidence: (0.78 - index as f64 * 0.04).max(0.62),
        })
        .collect()
}

fn build_duplicated_coverage(metrics: &[DistrictMetric]) -> Vec<DuplicatedCoverage> {
    let mut dense: Vec<&DistrictMetric> = metrics
        .iter()
        .filter(|item| item.score >= 50.0 || item.connectivity_score >= 55.0)
        .collect();
    dense.sort_by(|a, b| b.connectivity_score.total_cmp(&a.connectivity_score));

    dense.into_iter()
        .take(3)
        .map(|item| DuplicatedCoverage {
            corridor: format!("{} central corridor", item.name),
            problem: "too many overlapping routes".to_string(),
            action: "redistribute part of trips to underserved sector".to_string(),
            severity: if item.score > 65.0 { "medium" } else { "low" }.to_string(),
        })
        .collect()
}

fn build_comparison(metrics: &[DistrictMetric]) -> Vec<RouteComparison> {
    metrics
        .iter()
        .map(|item| {
            let weak = (70.0 - item.score).max(0.0);
            let time_saved = (weak / 2.2 + item.avg_time_to_hub_min / 5.0).round() as i32;
            let projected = (item.score + time_saved as f64 * 0.7).min(100.0).round() as i32;
            RouteComparison {
                district: item.name.clone(),
                current_score: item.score.round() as i32,
                projected_score: projected,
                time_saved_min: time_saved,
                affected_poi: item.poi_density.round() as i32,
                priority: priority_for(item.score).to_string(),
            }
        })
        .collect()
}

fn build_matrix(metrics: &[DistrictMetric]) -> Vec<RouteMatrixCell> {
    metrics
        .iter()
        .flat_map(|origin| {
            metrics.iter().filter_map(move |destination| {
                if origin.name == destination.name {
                    return None;
                }

                let current = estimate_connection_time(origin, destination);
                let target = estimate_target_time(origin, destination);
                let gap = (current - target).max(0);
                Some(RouteMatrixCell {
                    origin: origin.name.clone(),
                    destination: destination.name.clone(),
                    current_time_min: current,
                    target_time_min: target,
                    gap_min: gap,
                    priority: if gap >= 24 {
                        "high"
                    } else if gap >= 14 {
                        "medium"
                    } else {
                        "low"
                    }
                    .to_string(),
                })
            })
        })
        .collect()
}

fn find_metric<'a>(metrics: &'a [DistrictMetric], name: &str) -> Option<&'a DistrictMetric> {
    metrics
        .iter()
        .find(|item| item.name.eq_ignore_ascii_case(name.trim()))
}

fn estimate_connection_time(origin: &DistrictMetric, destination: &DistrictMetric) -> i32 {
    let distance_km = haversine_km(origin.lat, origin.lon, destination.lat, destination.lon);
    let weak_penalty = (100.0 - origin.score.min(destination.score)) / 3.2;
    let hub_penalty = (origin.avg_time_to_hub_min + destination.avg_time_to_hub_min) / 6.0;
    (distance_km * 3.2 + weak_penalty + hub_penalty + 14.0).round() as i32
}

fn estimate_target_time(origin: &DistrictMetric, destination: &DistrictMetric) -> i32 {
    let distance_km = haversine_km(origin.lat, origin.lon, destination.lat, destination.lon);
    (distance_km * 2.1 + 20.0).round().clamp(25.0, 42.0) as i32
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

fn improvement_for(intervention: &str) -> i32 {
    let normalized = intervention.to_ascii_lowercase();
    if normalized.contains("brt") {
        30
    } else if normalized.contains("hub") {
        24
    } else if normalized.contains("stop") {
        12
    } else {
        28
    }
}

fn suggested_via(index: usize) -> Vec<String> {
    match index {
        0 => vec!["Auezovsky".to_string()],
        1 => vec!["Bostandyk".to_string()],
        2 => vec!["Zhetysu".to_string()],
        _ => vec!["Almalinsky".to_string()],
    }
}

fn priority_for(score: f64) -> &'static str {
    if score < 40.0 {
        "high"
    } else if score < 60.0 {
        "medium"
    } else {
        "low"
    }
}

fn slug(value: &str) -> String {
    value.to_ascii_lowercase().replace(' ', "-")
}
