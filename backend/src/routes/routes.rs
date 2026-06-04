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
        .route("/api/routes/analyze", post(analyze_route))
        .route("/api/mobility/traffic", get(traffic_snapshot))
        .route(
            "/api/mobility/public-transport/locations",
            get(public_transport_locations),
        )
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

#[derive(Debug, Serialize, Clone)]
struct TrafficSegment {
    id: String,
    corridor: String,
    district: String,
    congestion_index: f64,
    average_speed_kmh: i32,
    delay_min: i32,
    source: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
struct PublicTransportVehicle {
    id: String,
    route_id: String,
    route_name: String,
    transport_type: String,
    lat: f64,
    lon: f64,
    occupancy: i32,
    delay_min: i32,
    bearing_deg: i32,
    source: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct RouteStopInput {
    name: String,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct AnalyzeRouteRequest {
    name: Option<String>,
    transport_type: Option<String>,
    stops: Vec<RouteStopInput>,
    frequency_min: Option<i32>,
    planned_vehicles: Option<i32>,
    greenfield: Option<bool>,
}

#[derive(Debug, Serialize)]
struct RouteCriterionScore {
    score: i32,
    level: String,
    summary: String,
    signals: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RouteAnalysisResponse {
    route_name: String,
    transport_type: String,
    total_distance_km: f64,
    estimated_duration_min: i32,
    city_need: RouteCriterionScore,
    duplication: RouteCriterionScore,
    overload_risk: RouteCriterionScore,
    recommendation: String,
    confidence: f64,
    data_sources: Vec<String>,
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

async fn traffic_snapshot(State(state): State<AppState>) -> Result<Json<Vec<TrafficSegment>>, String> {
    let metrics = load_district_metrics(&state).await?;
    Ok(Json(build_traffic_segments(&metrics)))
}

async fn public_transport_locations(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicTransportVehicle>>, String> {
    let metrics = load_district_metrics(&state).await?;
    let suggestions = build_suggestions(&metrics);
    Ok(Json(build_vehicle_locations(&metrics, &suggestions)))
}

async fn analyze_route(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeRouteRequest>,
) -> Result<Json<RouteAnalysisResponse>, String> {
    if payload.stops.len() < 2 {
        return Err("Route analysis needs at least two stops".to_string());
    }

    let metrics = load_district_metrics(&state).await?;
    if metrics.is_empty() {
        return Err("No district metrics available".to_string());
    }

    let traffic = build_traffic_segments(&metrics);
    let vehicles = build_vehicle_locations(&metrics, &build_suggestions(&metrics));
    let route_name = payload.name.unwrap_or_else(|| "New public transport route".to_string());
    let transport_type = payload.transport_type.unwrap_or_else(|| "bus".to_string());
    let frequency_min = payload.frequency_min.unwrap_or(10).clamp(3, 45);
    let planned_vehicles = payload.planned_vehicles.unwrap_or(8).clamp(1, 80);
    let greenfield = payload.greenfield.unwrap_or(false);

    let mut distance_km = 0.0;
    for pair in payload.stops.windows(2) {
        distance_km += haversine_km(pair[0].lat, pair[0].lon, pair[1].lat, pair[1].lon);
    }

    let covered_districts: Vec<&DistrictMetric> = metrics
        .iter()
        .filter(|district| {
            payload
                .stops
                .iter()
                .any(|stop| haversine_km(stop.lat, stop.lon, district.lat, district.lon) <= 5.5)
        })
        .collect();
    let covered = if covered_districts.is_empty() {
        vec![nearest_metric(&metrics, payload.stops[0].lat, payload.stops[0].lon)]
    } else {
        covered_districts
    };

    let avg_score = covered.iter().map(|item| item.score).sum::<f64>() / covered.len() as f64;
    let avg_hub_time =
        covered.iter().map(|item| item.avg_time_to_hub_min).sum::<f64>() / covered.len() as f64;
    let total_population = covered
        .iter()
        .filter_map(|item| item.population)
        .sum::<i32>()
        .max(1);
    let route_traffic = average_route_traffic(&payload.stops, &traffic);
    let overlap = if greenfield {
        0.0
    } else {
        route_overlap_score(&payload.stops, &metrics, &vehicles)
    };
    let route_capacity_per_hour = planned_vehicles as f64 * (60.0 / frequency_min as f64) * 85.0;
    let likely_demand_per_hour = total_population as f64 * ((100.0 - avg_score).max(18.0) / 1000.0)
        + distance_km * 34.0
        + route_traffic * 120.0;

    let need_score = ((100.0 - avg_score) * 0.48
        + avg_hub_time * 1.15
        + (total_population as f64 / 14000.0).min(24.0)
        + route_traffic * 18.0)
        .round()
        .clamp(0.0, 100.0) as i32;
    let duplication_score = (overlap * 100.0).round().clamp(0.0, 100.0) as i32;
    let overload_score = ((likely_demand_per_hour / route_capacity_per_hour) * 72.0
        + route_traffic * 22.0
        + (frequency_min as f64 - 8.0).max(0.0) * 1.2)
        .round()
        .clamp(0.0, 100.0) as i32;

    let city_need = RouteCriterionScore {
        score: need_score,
        level: level_for_positive(need_score).to_string(),
        summary: if need_score >= 70 {
            "Route is strongly justified for underserved districts".to_string()
        } else if need_score >= 45 {
            "Route can help the network, but should be refined".to_string()
        } else {
            "Route has limited city-wide necessity in its current shape".to_string()
        },
        signals: vec![
            format!("Average accessibility score on corridor: {:.0}", avg_score),
            format!("Average time to transfer hub: {:.0} min", avg_hub_time),
            format!("Estimated covered population: {}", total_population),
        ],
    };
    let duplication = RouteCriterionScore {
        score: duplication_score,
        level: level_for_risk(duplication_score).to_string(),
        summary: if duplication_score >= 70 {
            "High overlap with existing strong corridors and active routes".to_string()
        } else if duplication_score >= 40 {
            "Partial duplication, useful if stops are shifted toward gaps".to_string()
        } else if greenfield {
            "Greenfield scenario: duplication penalty is disabled for a network designed from zero".to_string()
        } else {
            "Low duplication, the line covers a distinct corridor".to_string()
        },
        signals: vec![
            format!("Route overlap index: {:.2}", overlap),
            format!("Existing vehicles near corridor: {}", nearby_vehicle_count(&payload.stops, &vehicles)),
            format!(
                "Stop chain: {}",
                payload
                    .stops
                    .iter()
                    .map(|stop| stop.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            format!("Districts crossed: {}", covered.iter().map(|item| item.name.as_str()).collect::<Vec<_>>().join(", ")),
        ],
    };
    let overload_risk = RouteCriterionScore {
        score: overload_score,
        level: level_for_risk(overload_score).to_string(),
        summary: if overload_score >= 70 {
            "High overload risk: add capacity or reduce headway before launch".to_string()
        } else if overload_score >= 40 {
            "Moderate overload risk during peaks".to_string()
        } else {
            "Capacity looks acceptable for MVP assumptions".to_string()
        },
        signals: vec![
            format!("Estimated demand: {:.0} pax/hour", likely_demand_per_hour),
            format!("Planned capacity: {:.0} pax/hour", route_capacity_per_hour),
            format!("Traffic pressure index: {:.2}", route_traffic),
        ],
    };

    let recommendation = build_route_recommendation(need_score, duplication_score, overload_score);

    Ok(Json(RouteAnalysisResponse {
        route_name,
        transport_type,
        total_distance_km: (distance_km * 10.0).round() / 10.0,
        estimated_duration_min: (distance_km * 3.2 + route_traffic * 18.0 + 8.0).round() as i32,
        city_need,
        duplication,
        overload_risk,
        recommendation,
        confidence: 0.74,
        data_sources: vec![
            "PostGIS district accessibility metrics".to_string(),
            "Traffic API layer: live-provider ready, estimated fallback now".to_string(),
            "Public transport geolocation API layer: GTFS/AVL ready, estimated fallback now".to_string(),
        ],
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

fn build_traffic_segments(metrics: &[DistrictMetric]) -> Vec<TrafficSegment> {
    let now = chrono::Utc::now().to_rfc3339();
    metrics
        .iter()
        .map(|item| {
            let pressure = ((100.0 - item.score) / 100.0 * 0.52
                + item.avg_time_to_hub_min / 60.0 * 0.28
                + item.poi_density.min(80.0) / 80.0 * 0.20)
                .clamp(0.08, 0.96);
            TrafficSegment {
                id: format!("traffic-{}", slug(&item.name)),
                corridor: format!("{} main corridor", item.name),
                district: item.name.clone(),
                congestion_index: (pressure * 100.0).round() / 100.0,
                average_speed_kmh: (42.0 - pressure * 24.0).round().max(8.0) as i32,
                delay_min: (pressure * 18.0).round() as i32,
                source: "estimated-fallback: connect live traffic provider here".to_string(),
                updated_at: now.clone(),
            }
        })
        .collect()
}

fn build_vehicle_locations(
    metrics: &[DistrictMetric],
    suggestions: &[SuggestedRoute],
) -> Vec<PublicTransportVehicle> {
    let now = chrono::Utc::now().to_rfc3339();
    suggestions
        .iter()
        .enumerate()
        .filter_map(|(index, route)| {
            let origin = find_metric(metrics, &route.origin)?;
            let destination = find_metric(metrics, &route.destination)?;
            let progress = 0.28 + (index as f64 * 0.17) % 0.55;
            Some(PublicTransportVehicle {
                id: format!("veh-{:02}", index + 1),
                route_id: route.id.clone(),
                route_name: route.name.clone(),
                transport_type: route.route_type.clone(),
                lat: origin.lat + (destination.lat - origin.lat) * progress,
                lon: origin.lon + (destination.lon - origin.lon) * progress,
                occupancy: (54 + index as i32 * 11).min(96),
                delay_min: (route.expected_impact / 8 + index as i32).min(14),
                bearing_deg: bearing_deg(origin.lat, origin.lon, destination.lat, destination.lon),
                source: "estimated-fallback: connect AVL/GTFS realtime feed here".to_string(),
                updated_at: now.clone(),
            })
        })
        .collect()
}

fn nearest_metric(metrics: &[DistrictMetric], lat: f64, lon: f64) -> &DistrictMetric {
    metrics
        .iter()
        .min_by(|a, b| {
            haversine_km(lat, lon, a.lat, a.lon).total_cmp(&haversine_km(lat, lon, b.lat, b.lon))
        })
        .expect("metrics are checked before nearest_metric")
}

fn average_route_traffic(stops: &[RouteStopInput], traffic: &[TrafficSegment]) -> f64 {
    if stops.is_empty() || traffic.is_empty() {
        return 0.35;
    }

    let sum = stops
        .iter()
        .map(|stop| {
            traffic
                .iter()
                .map(|segment| {
                    let pseudo_lat = match segment.district.as_str() {
                        "Almalinsky" => 43.2489,
                        "Auezovsky" => 43.2327,
                        "Bostandyk" => 43.2034,
                        "Nauryzbay" => 43.1972,
                        "Turksib" => 43.3335,
                        "Medeu" => 43.2244,
                        "Zhetysu" => 43.2901,
                        _ => 43.3006,
                    };
                    let pseudo_lon = match segment.district.as_str() {
                        "Almalinsky" => 76.9286,
                        "Auezovsky" => 76.8477,
                        "Bostandyk" => 76.9067,
                        "Nauryzbay" => 76.7825,
                        "Turksib" => 76.9870,
                        "Medeu" => 76.9958,
                        "Zhetysu" => 76.9350,
                        _ => 76.8287,
                    };
                    let distance = haversine_km(stop.lat, stop.lon, pseudo_lat, pseudo_lon);
                    segment.congestion_index / (1.0 + distance / 4.0)
                })
                .fold(0.0, f64::max)
        })
        .sum::<f64>();

    (sum / stops.len() as f64).clamp(0.08, 0.96)
}

fn route_overlap_score(
    stops: &[RouteStopInput],
    metrics: &[DistrictMetric],
    vehicles: &[PublicTransportVehicle],
) -> f64 {
    let strong_district_overlap = stops
        .iter()
        .filter(|stop| {
            let nearest = nearest_metric(metrics, stop.lat, stop.lon);
            nearest.score >= 62.0 || nearest.connectivity_score >= 58.0
        })
        .count() as f64
        / stops.len().max(1) as f64;
    let vehicle_overlap = nearby_vehicle_count(stops, vehicles) as f64 / 8.0;
    (strong_district_overlap * 0.62 + vehicle_overlap.min(1.0) * 0.38).clamp(0.0, 1.0)
}

fn nearby_vehicle_count(stops: &[RouteStopInput], vehicles: &[PublicTransportVehicle]) -> usize {
    vehicles
        .iter()
        .filter(|vehicle| {
            stops
                .iter()
                .any(|stop| haversine_km(stop.lat, stop.lon, vehicle.lat, vehicle.lon) <= 3.2)
        })
        .count()
}

fn level_for_positive(score: i32) -> &'static str {
    if score >= 70 {
        "high"
    } else if score >= 45 {
        "medium"
    } else {
        "low"
    }
}

fn level_for_risk(score: i32) -> &'static str {
    if score >= 70 {
        "high"
    } else if score >= 40 {
        "medium"
    } else {
        "low"
    }
}

fn build_route_recommendation(need: i32, duplication: i32, overload: i32) -> String {
    if need >= 70 && duplication < 55 && overload < 65 {
        "Launch as a pilot route and monitor demand during peak hours.".to_string()
    } else if need >= 60 && duplication >= 55 {
        "Route is useful, but shift stops away from duplicated corridors before launch.".to_string()
    } else if overload >= 70 {
        "Increase planned vehicles or reduce headway before approving the route.".to_string()
    } else if need < 45 {
        "Do not prioritize this route yet; test a feeder or on-demand service first.".to_string()
    } else {
        "Approve for scenario modelling, then refine the corridor using live traffic and AVL data.".to_string()
    }
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

fn bearing_deg(origin_lat: f64, origin_lon: f64, destination_lat: f64, destination_lon: f64) -> i32 {
    let lat1 = origin_lat.to_radians();
    let lat2 = destination_lat.to_radians();
    let d_lon = (destination_lon - origin_lon).to_radians();
    let y = d_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
    ((y.atan2(x).to_degrees() + 360.0) % 360.0).round() as i32
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
