use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
};

pub struct MongoCrud<T>
where
    T: Send + Sync,
{
    collection: Collection<T>,
}

impl<T> MongoCrud<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Unpin + Send + Sync,
{
    pub async fn new(db_name: &str, collection_name: &str, uri: &str) -> DbClassResult<Self> {
        let client_options =
            ClientOptions::parse(uri)
                .await
                .map_err(|err| DbClassError::CanNotDoAction {
                    error: err.to_string(),
                    action: "parse MongoDB URI".to_string(),
                    how_fix_it: "Ensure the MongoDB URI is correctly formatted".to_string(),
                })?;
        let client = Client::with_options(client_options).map_err(|err| DbClassError::from(err))?;
        let database = client.database(db_name);
        let collection = database.collection::<T>(collection_name);
        Ok(Self { collection })
    }

    pub async fn create(&self, document: T) -> DbClassResult<ObjectId> {
        let insert_result = self.collection.insert_one(document).await;
        match insert_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                action: "create".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
            Ok(id) => Ok(change_insertoneresult_into_object_id(id)),
        }
    }

    pub async fn get_one(&self, id: ObjectId, collection: Option<String>) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let item = self.collection.find_one(filter).await;

        match item {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "get on".to_string(),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                action: "get one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn get_many(&self) -> DbClassResult<Vec<T>> {
        let cursor = self
            .collection
            .find(doc! {})
            .await
            .map_err(|err| DbClassError::from(err))?;
        let items: Vec<T> = cursor
            .try_collect()
            .await
            .map_err(|err| DbClassError::from(err))?;
        Ok(items)
    }

    pub async fn update(&self, id: &str, updated_doc: T) -> DbClassResult<bool> {
        let object_id = ObjectId::parse_str(id).map_err(|_| DbClassError::InvalidId)?;
        let filter = doc! { "_id": object_id };
        let update_doc = doc! { "$set": mongodb::bson::to_document(&updated_doc).map_err(|err| DbClassError::from(mongodb::error::Error::from(err)))? };
        let update_result = self
            .collection
            .update_one(filter, update_doc)
            .await
            .map_err(|err| DbClassError::from(err))?;
        Ok(update_result.matched_count > 0)
    }

    pub async fn delete(&self, id: &str) -> DbClassResult<bool> {
        let object_id = ObjectId::parse_str(id).map_err(|_| DbClassError::InvalidId)?;
        let filter = doc! { "_id": object_id };
        let delete_result = self
            .collection
            .delete_one(filter)
            .await
            .map_err(|err| DbClassError::from(err))?;
        Ok(delete_result.deleted_count > 0)
    }
}
