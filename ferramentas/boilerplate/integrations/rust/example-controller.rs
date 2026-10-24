pub async fn controller()
-> Result<axum::Json<serde_json::Value>, (axum::http::StatusCode, &'static str)> {
    super::service::get().await.map(axum::Json).map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Dependência indisponível ou banco não preparado",
        )
    })
}
// @boilerplate:write
