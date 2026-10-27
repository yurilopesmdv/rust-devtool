pub mod modules;
use axum::{Json, Router, routing::get};
use serde_json::json;
use tower_http::cors::CorsLayer;
pub fn app() -> Router {
    let origin =
        std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:{{WEB_PORT}}".into());
    Router::new()
        .route("/health", get(|| async { Json(json!({"status":"ok"})) }))
        .route("/ready", get(|| async { Json(json!({"status":"ok"})) }))
        .route("/api/v1/example", get(modules::example::controller::controller))
        .layer(
            CorsLayer::new().allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("FRONTEND_ORIGIN inválida"),
            ),
        )
}
