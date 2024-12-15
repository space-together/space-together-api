use super::{
    class_db::{
        activities_db::ActivityDb, activities_type_db::ActivitiesTypeDb, class_db_db::ClassDb,
        class_group_db::ClassGroupDb,
    },
    conversation_db::message_db::MessageDb,
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};
use crate::{
    error::db_error::{DbError, DbResult},
    libs::{
        db::conversation_db::conversation_db_db::ConversationDb, functions::bytes_fn::format_bytes,
    },
    models::database_model::collection_model::{CollectionStats, DatabaseStats},
};
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{bson::doc, Client};
use serde_json::to_string;
use std::env;

#[derive(Debug)]
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: ClassDb,
    pub class_group: ClassGroupDb,
    pub conversation: ConversationDb,
    pub message: MessageDb,
    pub activities_type: ActivitiesTypeDb,
    pub activity: ActivityDb,
    pub stats: Option<DatabaseStats>,
}

impl ConnDb {
    pub async fn init() -> DbResult<Self> {
        dotenv().ok();
        let bd_uri = match env::var("MONGODB_URI") {
            Ok(val) => val.to_string(),
            Err(_) => "mongodb://localhost:27017/".to_string(),
        };

        let client = Client::with_uri_str(bd_uri).await;

        match client {
            Ok(res) => {
                let st_data = res.database("space-together-data");

                // Retrieve stats for collections
                let stats_result = Self::get_database_stats(&res, "space-together-data").await;

                // Initialize collections
                let user_role = UserRoleDb {
                    role: st_data.collection("user_role"),
                };
                let user = UserDb {
                    user: st_data.collection("users"),
                };
                let class = ClassDb {
                    class: st_data.collection("classes"),
                };
                let class_group = ClassGroupDb {
                    class_group: st_data.collection("class_groups"),
                };
                let conversation = ConversationDb {
                    conversation: st_data.collection("conversations"),
                };
                let message = MessageDb {
                    message: st_data.collection("messages"),
                };
                let activities_type = ActivitiesTypeDb {
                    activities_type: st_data.collection("activities_type"),
                };
                let activity = ActivityDb {
                    activity: st_data.collection("activities"),
                };

                let stats = match stats_result {
                    Ok(s) => Some(s),
                    Err(err) => {
                        println!("Failed to retrieve database stats: 😡{}😡", err);
                        None
                    }
                };

                println!("Database status: {:?}", stats);
                println!("Database connected successfully 🌼");

                Ok(Self {
                    user_role,
                    user,
                    class,
                    class_group,
                    conversation,
                    message,
                    activities_type,
                    activity,
                    stats,
                })
            }
            Err(err) => Err(DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }),
        }
    }

    /// Retrieve statistics for all collections in the database
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

            // Fetch all documents
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
                collection_size += doc_json.len(); // Calculate size
            }

            // Aggregate results
            total_documents += document_count;
            total_size_bytes += collection_size;

            collections_stats.push(CollectionStats {
                name: name.clone(),
                document_count,
                size_bytes: format_bytes(collection_size), // Format size
            });
        }

        Ok(DatabaseStats {
            total_documents,
            total_size_bytes: format_bytes(total_size_bytes),
            collections: collections_stats,
        })
    }
}
