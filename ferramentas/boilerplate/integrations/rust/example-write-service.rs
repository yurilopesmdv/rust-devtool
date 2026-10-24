pub async fn write(message: &str) -> Result<serde_json::Value, crate::infrastructure::Failure> {
    repositories::write(message).await?;
    // @boilerplate:invalidate
    Ok(serde_json::json!({"message":message}))
}
