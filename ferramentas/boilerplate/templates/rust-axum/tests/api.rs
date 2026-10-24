use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use {{RUST_CRATE}}::app;
use tower::ServiceExt;
#[tokio::test]
async fn health_and_example() {
    for path in ["/health", "/api/v1/example"] {
        let response = app()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
