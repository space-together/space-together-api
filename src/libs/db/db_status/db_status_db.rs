use futures::TryStreamExt;
use mongodb::{bson::doc, Client};
use serde_json::to_string;

use crate::{
    error::db_error::{DbError, DbResult},
    libs::functions::bytes_fn::format_bytes,
    models::database_model::collection_model::{CollectionStats, DatabaseStats},
};

pub async fn get_database_stats(client: &Client, db_name: &str) -> DbResult<DatabaseStats> {
    let database = client.database(db_name);
    let mut total_documents = 0;
    let mut total_size_bytes = 0;
    let mut collections_stats = Vec::new();

    let collection_names = match database.list_collection_names().await {
        Ok(names) => names,
        Err(err) => {
            return Err(DbError::CanNotGetAllTables {
                err: err.to_string(),
            });
        }
    };

    for name in &collection_names {
        let collection = database.collection::<mongodb::bson::Document>(name);

        let mut cursor = match collection.find(doc! {}).await {
            Ok(c) => c,
            Err(err) => {
                eprintln!(
                    "Error fetching documents from collection '{}': {:?}",
                    name, err
                );
                continue;
            }
        };

        let mut document_count = 0;
        let mut collection_size = 0;

        while let Some(doc) = cursor.try_next().await.unwrap_or_else(|err| {
            eprintln!(
                "Error reading document from collection '{}': {:?}",
                name, err
            );
            None
        }) {
            document_count += 1;
            let doc_json = match to_string(&doc) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!(
                        "Error serializing document from collection '{}': {:?}",
                        name, err
                    );
                    continue;
                }
            };
            collection_size += doc_json.len();
        }

        // Aggregate results
        total_documents += document_count;
        total_size_bytes += collection_size;

        collections_stats.push(CollectionStats {
            name: name.clone(),
            document_count,
            size_bytes: format_bytes(collection_size),
        });
    }

    Ok(DatabaseStats {
        total_documents,
        total_collection: collection_names.len(),
        total_size_bytes: format_bytes(total_size_bytes),
        collections: collections_stats,
    })
}
