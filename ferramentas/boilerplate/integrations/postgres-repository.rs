use crate::infrastructure::{Failure, database::pool};
pub async fn read() -> Result<String, Failure> {
    sqlx::query_scalar::<_, String>("SELECT message FROM examples WHERE key=$1")
        .bind("default")
        .fetch_optional(pool()?)
        .await?
        .ok_or_else(|| "Execute prepare".into())
}
pub async fn write(message: &str) -> Result<(), Failure> {
    sqlx::query("INSERT INTO examples(key,message) VALUES($1,$2) ON CONFLICT(key) DO UPDATE SET message=EXCLUDED.message").bind("default").bind(message).execute(pool()?).await?;
    Ok(())
}
