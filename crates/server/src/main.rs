mod config;
mod routes;

use std::net::SocketAddr;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = config::Config::from_env();
    let app = build_app();
    let addr: SocketAddr = ([0, 0, 0, 0], config.port).into();

    tracing::info!("kitty-cad-backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_app() -> Router {
    let cors = CorsLayer::very_permissive();

    Router::new()
        .merge(routes::meta::router())
        .merge(routes::users::router())
        .merge(routes::modeling::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
