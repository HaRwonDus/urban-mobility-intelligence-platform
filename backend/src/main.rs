mod routes;
mod services;
mod state;

use anyhow::Context;
use axum::Router;
use dotenvy::dotenv;
use state::AppState;
use std::{env, net::SocketAddr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let state = AppState::new(database_url).await?;

    let app = Router::new()
        .merge(routes::health::router())
        .merge(routes::objects::router())
        .merge(routes::accessibility::router())
        .merge(routes::recommendations::router())
        .merge(routes::routes::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = env::var("API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_string())
        .parse()
        .context("API_ADDR must be a valid socket address")?;

    tracing::info!("urban mobility API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
