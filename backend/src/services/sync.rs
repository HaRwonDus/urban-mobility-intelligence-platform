use crate::{models::SyncResponse, state::AppState};
use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

struct CollectorQuery {
    query: &'static str,
    object_type: &'static str,
    stop_kind: Option<&'static str>,
}

const COLLECTOR_QUERIES: &[CollectorQuery] = &[
    CollectorQuery {
        query: "bus stop",
        object_type: "stop",
        stop_kind: Some("stop"),
    },
    CollectorQuery {
        query: "metro",
        object_type: "metro",
        stop_kind: Some("metro"),
    },
    CollectorQuery {
        query: "hospital",
        object_type: "hospital",
        stop_kind: None,
    },
    CollectorQuery {
        query: "school",
        object_type: "school",
        stop_kind: None,
    },
    CollectorQuery {
        query: "shopping mall",
        object_type: "mall",
        stop_kind: None,
    },
    CollectorQuery {
        query: "airport",
        object_type: "airport",
        stop_kind: Some("airport"),
    },
    CollectorQuery {
        query: "railway station",
        object_type: "station",
        stop_kind: Some("station"),
    },
    CollectorQuery {
        query: "bus station",
        object_type: "bus_station",
        stop_kind: Some("bus_station"),
    },
    CollectorQuery {
        query: "university",
        object_type: "university",
        stop_kind: None,
    },
];

pub async fn sync_2gis(state: &AppState) -> Result<SyncResponse> {
    let log_id = create_sync_log(state).await?;
    let result = sync_2gis_inner(state).await;

    match result {
        Ok(response) => {
            finish_sync_log(
                state,
                log_id,
                "ok",
                response.objects_loaded,
                response.districts_updated,
                None,
            )
            .await?;
            Ok(response)
        }
        Err(err) => {
            finish_sync_log(state, log_id, "error", 0, 0, Some(err.to_string())).await?;
            Err(err)
        }
    }
}

async fn sync_2gis_inner(state: &AppState) -> Result<SyncResponse> {
    let mut objects_loaded = 0;

    for collector in COLLECTOR_QUERIES {
        let items = state
            .dgis
            .search_places(collector.query, Some("Almaty"), None)
            .await
            .with_context(|| format!("2GIS query failed: {}", collector.query))?;

        let mut tx = state.db.begin().await?;
        for item in items {
            if let Some(city_object_id) = upsert_city_object(&mut tx, collector, item)
                .await
                .context("failed to upsert city object")?
            {
                if let Some(stop_kind) = collector.stop_kind {
                    upsert_transport_stop(&mut tx, city_object_id, stop_kind).await?;
                }
                objects_loaded += 1;
            }
        }
        tx.commit().await?;
    }

    assign_nearest_districts(state).await?;
    let districts_updated = recalculate_scores(state).await?;
    generate_recommendations(state).await?;

    Ok(SyncResponse {
        status: "ok".to_string(),
        objects_loaded,
        districts_updated,
    })
}

async fn upsert_city_object(
    tx: &mut Transaction<'_, Postgres>,
    collector: &CollectorQuery,
    item: Value,
) -> Result<Option<Uuid>> {
    let Some((lat, lon)) = extract_point(&item) else {
        return Ok(None);
    };

    let name = item
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| item.get("full_name").and_then(Value::as_str))
        .unwrap_or(collector.query)
        .to_string();
    let address = item
        .pointer("/address/name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let external_id = item
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}:{:.6}:{:.6}:{}", collector.object_type, lat, lon, name));

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO city_objects (external_id, name, type, lat, lon, address, source, raw)
        VALUES ($1, $2, $3::city_object_type, $4, $5, $6, '2gis', $7)
        ON CONFLICT (source, external_id) DO UPDATE SET
          name = EXCLUDED.name,
          type = EXCLUDED.type,
          lat = EXCLUDED.lat,
          lon = EXCLUDED.lon,
          address = EXCLUDED.address,
          raw = EXCLUDED.raw,
          updated_at = now()
        RETURNING id
        "#,
    )
    .bind(external_id)
    .bind(name)
    .bind(collector.object_type)
    .bind(lat)
    .bind(lon)
    .bind(address)
    .bind(item)
    .fetch_one(&mut **tx)
    .await?;

    Ok(Some(id))
}

async fn upsert_transport_stop(
    tx: &mut Transaction<'_, Postgres>,
    city_object_id: Uuid,
    stop_kind: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO transport_stops (city_object_id, stop_kind)
        VALUES ($1, $2)
        ON CONFLICT (city_object_id) DO UPDATE SET stop_kind = EXCLUDED.stop_kind
        "#,
    )
    .bind(city_object_id)
    .bind(stop_kind)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn extract_point(item: &Value) -> Option<(f64, f64)> {
    let lat = item.pointer("/point/lat").and_then(Value::as_f64)?;
    let lon = item.pointer("/point/lon").and_then(Value::as_f64)?;
    Some((lat, lon))
}

async fn assign_nearest_districts(state: &AppState) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE city_objects co
        SET district_id = (
          SELECT d.id
          FROM districts d
          ORDER BY co.geom <-> d.geom
          LIMIT 1
        )
        WHERE co.source = '2gis'
        "#,
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

async fn recalculate_scores(state: &AppState) -> Result<i32> {
    let rows = sqlx::query(
        r#"
        WITH metrics AS (
          SELECT
            d.id AS district_id,
            d.name AS district,
            COALESCE(nearest_stop.distance_m / 80.0, 30.0) AS avg_time_to_stop_min,
            COALESCE(nearest_hub.distance_m / 80.0, 35.0) AS avg_time_to_hub_min,
            COALESCE(poi.poi_density, 0.0) AS poi_density,
            LEAST(100.0, COALESCE(important.important_count, 0.0) * 8.0) AS connectivity_score
          FROM districts d
          LEFT JOIN LATERAL (
            SELECT ST_Distance(d.geom, o.geom)::float8 AS distance_m
            FROM city_objects o
            WHERE o.type::text IN ('stop', 'metro', 'station', 'hub', 'bus_station', 'airport')
              AND ST_DWithin(d.geom, o.geom, 15000)
            ORDER BY ST_Distance(d.geom, o.geom)
            LIMIT 1
          ) nearest_stop ON true
          LEFT JOIN LATERAL (
            SELECT ST_Distance(d.geom, o.geom)::float8 AS distance_m
            FROM city_objects o
            WHERE o.type::text IN ('metro', 'station', 'hub', 'bus_station', 'airport')
              AND ST_DWithin(d.geom, o.geom, 20000)
            ORDER BY ST_Distance(d.geom, o.geom)
            LIMIT 1
          ) nearest_hub ON true
          LEFT JOIN LATERAL (
            SELECT COUNT(*)::float8 AS poi_density
            FROM city_objects o
            WHERE o.type::text IN ('school', 'mall', 'hospital', 'university')
              AND ST_DWithin(d.geom, o.geom, 5000)
          ) poi ON true
          LEFT JOIN LATERAL (
            SELECT COUNT(*)::float8 AS important_count
            FROM city_objects o
            WHERE o.type::text IN ('school', 'hospital', 'university', 'station', 'airport')
              AND ST_DWithin(d.geom, o.geom, 7000)
          ) important ON true
        ),
        scored AS (
          SELECT
            district_id,
            district,
            avg_time_to_stop_min,
            avg_time_to_hub_min,
            poi_density,
            connectivity_score,
            (
              0.40 * GREATEST(0.0, 100.0 - avg_time_to_stop_min * 5.0) +
              0.25 * GREATEST(0.0, 100.0 - avg_time_to_hub_min * 3.33) +
              0.20 * LEAST(100.0, poi_density * 5.0) +
              0.15 * connectivity_score
            ) AS score
          FROM metrics
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
            LEAST(100.0, GREATEST(0.0, score))
          FROM scored
          RETURNING district_id
        )
        INSERT INTO accessibility_scores (district, avg_time_to_stop, avg_time_to_hub, score)
        SELECT
          district,
          ROUND(avg_time_to_stop_min)::int,
          ROUND(avg_time_to_hub_min)::int,
          LEAST(100.0, GREATEST(0.0, score))
        FROM scored
        "#,
    )
    .execute(&state.db)
    .await?;

    Ok(rows.rows_affected() as i32)
}

async fn generate_recommendations(state: &AppState) -> Result<()> {
    sqlx::query("DELETE FROM ai_recommendations WHERE model_name = 'rules-v2'")
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"
        WITH latest AS (
          SELECT DISTINCT ON (ms.district_id)
            ms.*,
            d.name AS district
          FROM mobility_scores ms
          JOIN districts d ON d.id = ms.district_id
          ORDER BY ms.district_id, ms.calculated_at DESC
        )
        INSERT INTO ai_recommendations (district_id, area, problem, recommendation, confidence, model_name)
        SELECT
          district_id,
          district,
          'High POI density with weak stop access',
          'Add a new stop cluster or express feeder route near the strongest POI concentration.',
          0.82,
          'rules-v2'
        FROM latest
        WHERE avg_time_to_stop_min > 15 AND poi_density >= 8
        UNION ALL
        SELECT
          district_id,
          district,
          'Long access time to metro or transfer hub',
          'Evaluate a transfer hub or trunk-route connection for this district.',
          0.76,
          'rules-v2'
        FROM latest
        WHERE avg_time_to_hub_min > 20
        UNION ALL
        SELECT
          district_id,
          district,
          'Important objects have weak transport connectivity',
          'Prioritize hub placement around hospitals, schools, universities, or stations.',
          0.71,
          'rules-v2'
        FROM latest
        WHERE connectivity_score < 45 AND poi_density >= 4
        "#,
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

async fn create_sync_log(state: &AppState) -> Result<Uuid> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO sync_logs (status)
        VALUES ('running')
        RETURNING id
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(id)
}

async fn finish_sync_log(
    state: &AppState,
    id: Uuid,
    status: &str,
    objects_loaded: i32,
    districts_updated: i32,
    message: Option<String>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE sync_logs
        SET status = $2,
            objects_loaded = $3,
            districts_updated = $4,
            message = $5,
            finished_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(objects_loaded)
    .bind(districts_updated)
    .bind(message)
    .execute(&state.db)
    .await?;

    Ok(())
}
