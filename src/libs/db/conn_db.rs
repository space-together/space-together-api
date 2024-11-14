use std::env;

use dotenv::dotenv;
use mongodb::Client;

use crate::{
    error::db_error::{DbError, DbResult},
    libs::db::conversation_db::conversation_db_db::ConversationDb,
};

use super::{
    class_db::{class_db_db::ClassDb, class_group_db::ClassGroupDb},
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};

#[derive(Debug)]
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: ClassDb,
    pub class_group: ClassGroupDb,
    pub conversation: ConversationDb,
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
                // database 🌼
                let st_data = res.database("space-together-data");

                // collection 🏂🏽
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

                println!("Database connected successfully 🌼");

                Ok(Self {
                    user_role,
                    user,
                    class,
                    class_group,
                    conversation,
                })
            }
            Err(err) => Err(DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }),
        }
    }
}
