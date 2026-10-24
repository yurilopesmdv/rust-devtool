pub fn service() -> serde_json::Value {
    serde_json::json!({"message":"API {{PROJECT_NAME}} funcionando"})
}
pub async fn controller() -> axum::Json<serde_json::Value> {
    axum::Json(service())
}
