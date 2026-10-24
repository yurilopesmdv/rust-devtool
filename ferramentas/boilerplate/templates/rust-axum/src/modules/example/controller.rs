pub async fn controller() -> axum::Json<serde_json::Value> {
    axum::Json(super::service::get())
}
