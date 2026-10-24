pub async fn write_controller(
    axum::Json(body): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, (axum::http::StatusCode, &'static str)> {
    let message = body
        .get("message")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && s.chars().count() <= 200)
        .ok_or((
            axum::http::StatusCode::BAD_REQUEST,
            "message precisa conter 1 a 200 caracteres",
        ))?;
    super::service::write(message)
        .await
        .map(axum::Json)
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Falha de persistência",
            )
        })
}
