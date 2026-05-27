use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/routes", post(create_route))
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
