use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/ping", get(ping))
        .route("/", get(root))
}

async fn ping() -> Json<Value> {
    Json(json!({ "message": "pong" }))
}

async fn root() -> Json<Value> {
    Json(json!({
        "name": "kitty-cad-backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
