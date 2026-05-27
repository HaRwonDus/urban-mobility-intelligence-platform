use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/objects", get(list_objects))
        .route("/api/objects", get(list_objects))
        .route("/objects/search", post(search_objects))
        .route("/api/objects/search", post(search_objects))
}

#[derive(Debug, Deserialize)]
struct ListObjectsQuery {
    r#type: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
struct CityObject {
    id: Uuid,
    name: String,
    r#type: String,
    lat: f64,
    lon: f64,
    address: Option<String>,
    source: String,
}

async fn list_objects(
    State(state): State<AppState>,
    Query(query): Query<ListObjectsQuery>,
) -> Result<Json<Vec<CityObject>>, String> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);

    let rows = if let Some(object_type) = query.r#type {
        sqlx::query_as::<_, CityObject>(
            r#"
            SELECT id, name, type::text, lat, lon, address, source
            FROM city_objects
            WHERE type::text = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(object_type)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, CityObject>(
            r#"
            SELECT id, name, type::text, lat, lon, address, source
            FROM city_objects
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
struct SearchObjectsRequest {
    query: String,
    city: Option<String>,
    object_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct SearchObjectsResponse {
    provider: &'static str,
    query: String,
    items: Vec<serde_json::Value>,
}

async fn search_objects(
    State(state): State<AppState>,
    Json(payload): Json<SearchObjectsRequest>,
) -> Result<Json<SearchObjectsResponse>, String> {
    let items = state
        .dgis
        .search_places(
            &payload.query,
            payload.city.as_deref(),
            payload.object_type.as_deref(),
        )
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(SearchObjectsResponse {
        provider: "2gis",
        query: payload.query,
        items,
    }))
}
