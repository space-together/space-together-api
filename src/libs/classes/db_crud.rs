use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::ClientOptions,
    results::UpdateResult,
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
    pub async fn new(
        db_name: &str,
        collection_name: &str,
        uri: &str,
        collection: Option<String>,
    ) -> DbClassResult<Self> {
        let client_options =
            ClientOptions::parse(uri)
                .await
                .map_err(|err| DbClassError::CanNotDoAction {
                    error: err.to_string(),
                    collection: collection.unwrap_or_else(|| "unknown".to_string()),
                    action: "parse MongoDB URI".to_string(),
                    how_fix_it: "Ensure the MongoDB URI is correctly formatted".to_string(),
                })?;
        let client = Client::with_options(client_options).map_err(|err| DbClassError::from(err))?;
        let database = client.database(db_name);
        let collection = database.collection::<T>(collection_name);
        Ok(Self { collection })
    }

    pub async fn create(&self, document: T, collection: Option<String>) -> DbClassResult<ObjectId> {
        let insert_result = self.collection.insert_one(document).await;
        match insert_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
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
                action: "get one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "get one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn get_many(&self, collection: Option<String>) -> DbClassResult<Vec<T>> {
        let cursor_result = self.collection.find(doc! {}).await;
        match cursor_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "get many".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
            Ok(mut r) => {
                let items = r.try_collect().await;
                match items {
                    Ok(data) => Ok(data),
                    Err(err) => Err(DbClassError::CanNotDoAction {
                        error: err.to_string(),
                        collection: collection.unwrap_or_else(|| "unknown".to_string()),
                        action: "convert many into array".to_string(),
                        how_fix_it: "try again later".to_string(),
                    }),
                }
            }
        }
    }

    pub async fn update(
        &self,
        id: ObjectId,
        updated_doc: Document,
        collection: Option<String>,
    ) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let update_result = self
            .collection
            .find_one_and_update(filter, doc! {"$set" : updated_doc})
            .await;
        match update_result {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "update one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "update".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn delete(&self, id: ObjectId, collection: Option<String>) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let delete_result = self.collection.find_one_and_delete(filter).await;

        match delete_result {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "delete one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "delete one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }
}
