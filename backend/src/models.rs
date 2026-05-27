use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct District {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub lat: f64,
    pub lon: f64,
    pub population: Option<i32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DistrictScore {
    pub district_id: Uuid,
    pub district: String,
    pub avg_time_to_stop_min: f64,
    pub avg_time_to_hub_min: f64,
    pub poi_density: f64,
    pub connectivity_score: f64,
    pub score: f64,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Recommendation {
    pub id: Uuid,
    pub area: String,
    pub problem: String,
    pub recommendation: String,
    pub confidence: f64,
    pub model_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SyncLog {
    pub id: Uuid,
    pub provider: String,
    pub status: String,
    pub objects_loaded: i32,
    pub districts_updated: i32,
    pub message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub status: String,
    pub objects_loaded: i32,
    pub districts_updated: i32,
}
