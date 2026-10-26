use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use {{CRATE}}::{app, infrastructure};
use tower::ServiceExt;

async fn post(message: serde_json::Value) -> StatusCode {
    app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/example")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({"message": message}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn liveness_does_not_require_a_database() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "Requer DATABASE_URL e banco preparado; rode cargo test -- --ignored"]
async fn persistence() {
    use {{CRATE}}::modules::example::repositories;
    infrastructure::initialize().await.unwrap();
    infrastructure::schema::prepare().await.unwrap();
    let previous = repositories::read().await.unwrap();
    let mut results = Vec::new();
    for (message, expected) in [
        (serde_json::json!("😀".repeat(200)), StatusCode::OK),
        (serde_json::json!("😀".repeat(201)), StatusCode::BAD_REQUEST),
        (serde_json::json!(123), StatusCode::BAD_REQUEST),
    ] {
        let status = post(message.clone()).await;
        results.push((status, expected));
        if expected == StatusCode::OK {
            results.push((
                if repositories::read().await.unwrap() == message.as_str().unwrap() {
                    StatusCode::OK
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                },
                StatusCode::OK,
            ));
        }
    }
    // Restore before assertions, even when the observed HTTP status is incorrect.
    repositories::write(&previous).await.unwrap();
    let ready = app()
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status();
    infrastructure::close().await;
    assert_eq!(ready, StatusCode::OK);
    for (actual, expected) in results {
        assert_eq!(actual, expected);
    }
    for (path, status) in [
        ("/ready", StatusCode::SERVICE_UNAVAILABLE),
        ("/health", StatusCode::OK),
    ] {
        let response = app()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}
