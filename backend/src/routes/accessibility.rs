use crate::state::AppState;
use axum::{extract::State, routing::{get, post}, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accessibility", get(list_accessibility))
        .route("/api/accessibility", get(list_accessibility))
        .route("/api/accessibility/heatmap", get(accessibility_heatmap))
        .route("/api/accessibility/recalculate", post(recalculate_accessibility))
}

#[derive(Debug, Serialize, FromRow)]
struct AccessibilityScore {
    id: Uuid,
    district: String,
    avg_time_to_stop: i32,
    avg_time_to_hub: i32,
    score: f64,
    calculated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
struct HeatmapCell {
    district_id: Uuid,
    district: String,
    lat: f64,
    lon: f64,
    boundary_geojson: serde_json::Value,
    avg_time_to_stop_min: f64,
    avg_time_to_hub_min: f64,
    poi_density: f64,
    connectivity_score: f64,
    score: f64,
}

#[derive(Debug, Serialize)]
struct RecalculateResponse {
    districts_updated: u64,
}

async fn list_accessibility(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccessibilityScore>>, String> {
    let rows = sqlx::query_as::<_, AccessibilityScore>(
        r#"
        SELECT id, district, avg_time_to_stop, avg_time_to_hub, score::float8, calculated_at
        FROM accessibility_scores
        ORDER BY calculated_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}

async fn accessibility_heatmap(
    State(state): State<AppState>,
) -> Result<Json<Vec<HeatmapCell>>, String> {
    let rows = sqlx::query_as::<_, HeatmapCell>(
        r#"
        SELECT
          d.id AS district_id,
          d.name AS district,
          d.lat,
          d.lon,
          ST_AsGeoJSON(ST_Buffer(d.geom, 5500)::geometry)::jsonb AS boundary_geojson,
          COALESCE(ms.avg_time_to_stop_min, 18)::float8 AS avg_time_to_stop_min,
          COALESCE(ms.avg_time_to_hub_min, 28)::float8 AS avg_time_to_hub_min,
          COALESCE(ms.poi_density, 0)::float8 AS poi_density,
          COALESCE(ms.connectivity_score, 35)::float8 AS connectivity_score,
          COALESCE(ms.score, 45)::float8 AS score
        FROM districts d
        LEFT JOIN LATERAL (
          SELECT avg_time_to_stop_min, avg_time_to_hub_min, poi_density, connectivity_score, score
          FROM mobility_scores
          WHERE district_id = d.id
          ORDER BY calculated_at DESC
          LIMIT 1
        ) ms ON true
        ORDER BY score ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}

async fn recalculate_accessibility(
    State(state): State<AppState>,
) -> Result<Json<RecalculateResponse>, String> {
    let result = sqlx::query(
        r#"
        WITH district_metrics AS (
          SELECT
            d.id AS district_id,
            d.name AS district,
            COALESCE(nearest_stop.distance_m / 80.0, 30.0) AS avg_time_to_stop_min,
            COALESCE(nearest_hub.distance_m / 80.0, 35.0) AS avg_time_to_hub_min,
            COALESCE(poi.poi_density, 0.0) AS poi_density,
            LEAST(100.0, COALESCE(coverage.stop_count, 0.0) * 8.0) AS connectivity_score
          FROM districts d
          LEFT JOIN LATERAL (
            SELECT ST_Distance(d.geom, o.geom)::float8 AS distance_m
            FROM city_objects o
            WHERE o.type::text IN ('stop', 'metro', 'station', 'hub', 'bus_station')
              AND ST_DWithin(d.geom, o.geom, 15000)
            ORDER BY ST_Distance(d.geom, o.geom)
            LIMIT 1
          ) nearest_stop ON true
          LEFT JOIN LATERAL (
            SELECT ST_Distance(d.geom, o.geom)::float8 AS distance_m
            FROM city_objects o
            WHERE o.type::text IN ('metro', 'station', 'hub', 'bus_station')
              AND ST_DWithin(d.geom, o.geom, 20000)
            ORDER BY ST_Distance(d.geom, o.geom)
            LIMIT 1
          ) nearest_hub ON true
          LEFT JOIN LATERAL (
            SELECT COUNT(*)::float8 AS poi_density
            FROM city_objects o
            WHERE o.type::text IN ('school', 'mall', 'hospital', 'university')
              AND ST_DWithin(d.geom, o.geom, 5500)
          ) poi ON true
          LEFT JOIN LATERAL (
            SELECT COUNT(*)::float8 AS stop_count
            FROM city_objects o
            WHERE o.type::text IN ('stop', 'metro', 'station', 'hub', 'bus_station')
              AND ST_DWithin(d.geom, o.geom, 7000)
          ) coverage ON true
        ),
        scored AS (
          SELECT
            district_id,
            avg_time_to_stop_min,
            avg_time_to_hub_min,
            poi_density,
            connectivity_score,
            GREATEST(
              0.0,
              LEAST(
                100.0,
                100.0
                - avg_time_to_stop_min * 1.9
                - avg_time_to_hub_min * 1.15
                + connectivity_score * 0.32
                + LEAST(poi_density, 40.0) * 0.18
              )
            ) AS score
          FROM district_metrics
        ),
        inserted AS (
          INSERT INTO mobility_scores (
            district_id,
            avg_time_to_stop_min,
            avg_time_to_hub_min,
            poi_density,
            connectivity_score,
            score
          )
          SELECT
            district_id,
            avg_time_to_stop_min,
            avg_time_to_hub_min,
            poi_density,
            connectivity_score,
            score
          FROM scored
          RETURNING district_id
        )
        INSERT INTO accessibility_scores (district, avg_time_to_stop, avg_time_to_hub, score)
        SELECT
          district,
          ROUND(avg_time_to_stop_min)::int,
          ROUND(avg_time_to_hub_min)::int,
          score
        FROM scored
        "#,
    )
    .execute(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(RecalculateResponse {
        districts_updated: result.rows_affected(),
    }))
}
