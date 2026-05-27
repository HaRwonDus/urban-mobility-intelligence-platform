use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/accessibility", get(list_accessibility))
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
