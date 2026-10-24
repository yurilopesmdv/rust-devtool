use super::{Failure, database::pool};
pub async fn prepare() -> Result<(), Failure> {
    sqlx::query("CREATE TABLE IF NOT EXISTS examples(key TEXT PRIMARY KEY,message TEXT NOT NULL)")
        .execute(pool()?)
        .await?;
    sqlx::query("INSERT INTO examples(key,message) VALUES('default','API funcionando') ON CONFLICT DO NOTHING").execute(pool()?).await?;
    Ok(())
}
