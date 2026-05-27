use crate::{
    models::{District, DistrictScore},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/districts", get(list_districts))
        .route("/api/districts/:id/score", get(get_district_score))
}

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
