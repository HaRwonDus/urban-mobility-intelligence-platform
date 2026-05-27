use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/recommendations", get(list_recommendations))
}

#[derive(Debug, Serialize, FromRow)]
struct Recommendation {
    id: Uuid,
    area: String,
    problem: String,
    recommendation: String,
    confidence: f64,
    model_name: String,
    created_at: DateTime<Utc>,
}

async fn list_recommendations(
    State(state): State<AppState>,
) -> Result<Json<Vec<Recommendation>>, String> {
    let rows = sqlx::query_as::<_, Recommendation>(
        r#"
        SELECT id, area, problem, recommendation, confidence::float8, model_name, created_at
        FROM ai_recommendations
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}
