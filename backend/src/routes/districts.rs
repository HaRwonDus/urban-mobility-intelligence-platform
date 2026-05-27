use crate::{
    models::{District, DistrictScore},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/districts", get(list_districts))
        .route("/api/districts/:id/score", get(get_district_score))
        .route("/api/districts/:id/objects", get(list_district_objects))
        .route("/api/districts/:id/nearest-stops", get(list_nearest_stops))
}

const DISTRICT_RADIUS_M: i32 = 5_500;

async fn list_districts(State(state): State<AppState>) -> Result<Json<Vec<District>>, String> {
    let rows = sqlx::query_as::<_, District>(
        r#"
        SELECT id, name, slug, lat, lon, population
        FROM districts
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}

async fn get_district_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<DistrictScore>>, String> {
    let row = sqlx::query_as::<_, DistrictScore>(
        r#"
        SELECT
          ms.district_id,
          d.name AS district,
          ms.avg_time_to_stop_min::float8,
          ms.avg_time_to_hub_min::float8,
          ms.poi_density::float8,
          ms.connectivity_score::float8,
          ms.score::float8,
          ms.calculated_at
        FROM mobility_scores ms
        JOIN districts d ON d.id = ms.district_id
        WHERE ms.district_id = $1
        ORDER BY ms.calculated_at DESC
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(row))
}

#[derive(Debug, Serialize, FromRow)]
struct DistrictObject {
    id: Uuid,
    name: String,
    r#type: String,
    lat: f64,
    lon: f64,
    address: Option<String>,
    source: String,
    distance_m: f64,
}

#[derive(Debug, Serialize, FromRow)]
struct NearestStop {
    id: Uuid,
    name: String,
    stop_kind: String,
    route_count: i32,
    lat: f64,
    lon: f64,
    distance_m: f64,
}

#[derive(Debug, Deserialize)]
struct DistrictPath {
    id: Uuid,
}

async fn list_district_objects(
    State(state): State<AppState>,
    Path(path): Path<DistrictPath>,
) -> Result<Json<Vec<DistrictObject>>, String> {
    let rows = sqlx::query_as::<_, DistrictObject>(
        r#"
        WITH district_area AS (
          SELECT
            id,
            geom,
            ST_Buffer(geom, $2)::geometry AS boundary
          FROM districts
          WHERE id = $1
        )
        SELECT
          o.id,
          o.name,
          o.type::text AS type,
          o.lat,
          o.lon,
          o.address,
          o.source,
          ST_Distance(o.geom, d.geom)::float8 AS distance_m
        FROM city_objects o
        JOIN district_area d ON ST_Within(o.geom::geometry, d.boundary)
        ORDER BY distance_m
        LIMIT 500
        "#,
    )
    .bind(path.id)
    .bind(DISTRICT_RADIUS_M)
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}

async fn list_nearest_stops(
    State(state): State<AppState>,
    Path(path): Path<DistrictPath>,
) -> Result<Json<Vec<NearestStop>>, String> {
    let rows = sqlx::query_as::<_, NearestStop>(
        r#"
        WITH district_area AS (
          SELECT id, geom
          FROM districts
          WHERE id = $1
        )
        SELECT
          o.id,
          o.name,
          COALESCE(ts.stop_kind, o.type::text) AS stop_kind,
          COALESCE(ts.route_count, 0) AS route_count,
          o.lat,
          o.lon,
          ST_Distance(o.geom, d.geom)::float8 AS distance_m
        FROM district_area d
        JOIN city_objects o ON o.type IN ('stop', 'metro', 'station', 'hub', 'bus_station')
        LEFT JOIN transport_stops ts ON ts.city_object_id = o.id
        WHERE ST_DWithin(o.geom, d.geom, 15000)
        ORDER BY ST_Distance(o.geom, d.geom)
        LIMIT 25
        "#,
    )
    .bind(path.id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}
