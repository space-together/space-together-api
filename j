use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{bson::doc, Client};
use serde_json::to_string;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

// Necessary imports for your project
use crate::error::db_error::{DbError, DbResult};
use crate::libs::db::{
    conversation_db::message_db::MessageDb,
    request_db::{request_db_db::RequestDb, request_type_db::RequestTypeDb},
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};
use crate::models::database_model::collection_model::{CollectionStats, DatabaseStats};

use super::class_db::activities_db::ActivityDb;
use super::class_db::activities_type_db::ActivitiesTypeDb;
use super::class_db::class_db_db::ClassDb;
use super::class_db::class_group_db::ClassGroupDb;
use super::conversation_db::conversation_db_db::ConversationDb;

// Formatting utility
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut index = 0;
    while size > 1024.0 && index < UNITS.len() - 1 {
        size /= 1024.0;
        index += 1;
    }
    format!("{:.2} {}", size, UNITS[index])
}

// Core ConnDb Struct
#[derive(Debug)] // Automatically derives the Debug trait
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: ClassDb,
    pub class_group: ClassGroupDb,
    pub conversation: ConversationDb,
    pub message: MessageDb,
    pub activities_type: ActivitiesTypeDb,
    pub activity: ActivityDb,
    pub stats: Arc<RwLock<Option<DatabaseStats>>>, // Shared stats
    pub request_type: RequestTypeDb,
    pub request: RequestDb,
}

impl ConnDb {
    /// Initializes the database connection and collections
    pub async fn init() -> DbResult<Self> {
        dotenv().ok();
        let db_uri =
            env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017/".to_string());

        let client = Client::with_uri_str(&db_uri).await.map_err(|err| {
            DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }
        })?;
        let database_name = "space-together-data";
        let stats = Arc::new(RwLock::new(None)); // Initialize shared stats

        let st_data = client.database(database_name);

        // Initialize individual collections
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
        let request_type = RequestTypeDb {
            request: st_data.collection("request_type"),
        };
        let request = RequestDb {
            request: st_data.collection("requests"),
        };

        // Start stats update in background
        let client_arc = Arc::new(client.clone());
        let stats_clone = stats.clone();
        tokio::spawn(async move {
            Self::start_stats_updater(client_arc, database_name, stats_clone).await;
        });

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
            request_type,
            request,
        })
    }

    /// Starts background database stats updates
    async fn start_stats_updater(
        client: Arc<Client>,
        db_name: &str,
        stats: Arc<RwLock<Option<DatabaseStats>>>,
    ) {
        loop {
            match Self::get_database_stats(&client, db_name).await {
                Ok(new_stats) => {
                    let mut stats_lock = stats.write().await;
                    *stats_lock = Some(new_stats);
                }
                Err(err) => {
                    eprintln!("Failed to update database stats: {:?}", err);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // Update every 60 seconds
        }
    }

    /// Fetches database stats
    pub async fn get_database_stats(client: &Client, db_name: &str) -> DbResult<DatabaseStats> {
        let database = client.database(db_name);
        let mut total_documents = 0;
        let mut total_size_bytes = 0;
        let mut collections_stats = Vec::new();

        let collection_names =
            database
                .list_collection_names()
                .await
                .map_err(|err| DbError::CanNotGetAllTables {
                    err: err.to_string(),
                })?;

        for name in &collection_names {
            let collection = database.collection::<mongodb::bson::Document>(name);

            let mut cursor =
                collection
                    .find(doc! {})
                    .await
                    .map_err(|err| DbError::CannotReadDocuments {
                        err: err.to_string(),
                    })?;

            let mut document_count = 0;
            let mut collection_size = 0;

            while let Ok(Some(doc_result)) = cursor.try_next().await {
                match doc_result {
                    Ok(doc) => {
                        document_count += 1;
                        match to_string(&doc) {
                            Ok(doc_json) => {
                                collection_size += doc_json.len();
                            }
                            Err(err) => {
                                eprintln!(
                                    "Error serializing document from collection '{}': {:?}",
                                    name, err
                                );
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "Error reading document from collection '{}': {:?}",
                            name, err
                        );
                    }
                }
            }

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
            total_collection: collections_stats.len(),
            total_size_bytes: format_bytes(total_size_bytes),
            collections: collections_stats,
        })
    }
}
