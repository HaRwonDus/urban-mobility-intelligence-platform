use crate::{models::SyncLog, services, state::AppState};
use axum::{extract::State, routing::get, routing::post, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/sync/2gis", post(sync_2gis))
        .route("/api/sync/logs", get(list_sync_logs))
}

async fn sync_2gis(
    State(state): State<AppState>,
) -> Result<Json<crate::models::SyncResponse>, String> {
    let response = services::sync::sync_2gis(&state)
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(response))
}

async fn list_sync_logs(State(state): State<AppState>) -> Result<Json<Vec<SyncLog>>, String> {
    let rows = sqlx::query_as::<_, SyncLog>(
        r#"
        SELECT id, provider, status, objects_loaded, districts_updated, message, started_at, finished_at
        FROM sync_logs
        ORDER BY started_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    Ok(Json(rows))
}
