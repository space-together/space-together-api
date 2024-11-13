use std::env;

use dotenv::dotenv;
use mongodb::Client;

use crate::error::db_error::{DbError, DbResult};

use super::{
    class_db::class_db_db::ClassDb,
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};

#[derive(Debug)]
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: ClassDb,
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
                let st_data_db = res.database("space-together-data");

                // collection 🏂🏽
                let user_role_db = UserRoleDb {
                    role: st_data_db.collection("user_role"),
                };
                let user_db = UserDb {
                    user: st_data_db.collection("users"),
                };
                let class_db = ClassDb {
                    class: st_data_db.collection("classes"),
                };

                println!("Database connected successfully 🌼");

                Ok(Self {
                    user_role: user_role_db,
                    user: user_db,
                    class: class_db,
                })
            }
            Err(err) => Err(DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }),
        }
    }
}
