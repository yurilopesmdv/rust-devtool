use crate::infrastructure::{Failure, database::database};
use mongodb::bson::doc;
pub async fn read() -> Result<String, Failure> {
    let row = database()?
        .collection::<mongodb::bson::Document>("examples")
        .find_one(doc! {"_id":"default"})
        .await?
        .ok_or("Execute prepare")?;
    Ok(row.get_str("message")?.to_string())
}
pub async fn write(message: &str) -> Result<(), Failure> {
    database()?
        .collection::<mongodb::bson::Document>("examples")
        .update_one(doc! {"_id":"default"}, doc! {"$set":{"message":message}})
        .upsert(true)
        .await?;
    Ok(())
}
