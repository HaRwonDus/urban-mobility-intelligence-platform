mod config;
mod models;
mod routes;
mod services;
mod state;

use axum::Router;
use config::Config;
use dotenvy::dotenv;
use state::AppState;
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

    let config = Config::from_env()?;
    let addr = config.api_addr;
    let state = AppState::new(config).await?;

    let app = Router::new()
        .merge(routes::health::router())
        .merge(routes::objects::router())
        .merge(routes::accessibility::router())
        .merge(routes::recommendations::router())
        .merge(routes::routes::router())
        .merge(routes::districts::router())
        .merge(routes::sync::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("urban mobility API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
