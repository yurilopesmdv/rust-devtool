use super::{Failure, database::database};
use mongodb::bson::doc;
pub async fn prepare() -> Result<(), Failure> {
    let collection = database()?.collection::<mongodb::bson::Document>("examples");
    collection
        .create_index(
            mongodb::IndexModel::builder()
                .keys(doc! {"message":1})
                .build(),
        )
        .await?;
    collection
        .update_one(
            doc! {"_id":"default"},
            doc! {"$setOnInsert":{"message":"API funcionando"}},
        )
        .upsert(true)
        .await?;
    Ok(())
}
