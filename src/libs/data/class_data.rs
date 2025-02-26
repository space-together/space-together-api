use std::sync::Arc;

use mongodb::bson::doc;

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::schemas::main_class_schema::ClassRoomSchema,
    AppState,
};

pub async fn get_main_class_by_username(
    state: Arc<AppState>,
    username: &str,
) -> DbClassResult<ClassRoomSchema> {
    match state
        .db
        .main_class
        .collection
        .find_one(doc! {"username" : username})
        .await
    {
        Ok(Some(doc)) => Ok(doc),
        Ok(None) => Err(DbClassError::OtherError {
            err: "Main class username not found ,please use other username".to_string(),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Some thing went wrong to get main class name by username".to_string(),
        }),
    }
}
