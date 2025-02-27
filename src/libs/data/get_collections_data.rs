use std::sync::Arc;

use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::schemas::education_schema::EducationSchema,
    AppState,
};

pub async fn service_get_education_by_username(
    state: Arc<AppState>,
    username: &str,
) -> DbClassResult<EducationSchema> {
    match state
        .db
        .educations
        .collection
        .find_one(doc! {"username" : username})
        .await
    {
        Ok(Some(doc)) => Ok(doc),
        Ok(None) => Err(DbClassError::OtherError {
            err: "username not found ,please use other username".to_string(),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Some thing went wrong to item by username".to_string(),
        }),
    }
}

pub async fn service_get_education_by_id(
    state: Arc<AppState>,
    id: &ObjectId,
) -> DbClassResult<EducationSchema> {
    match state
        .db
        .educations
        .collection
        .find_one(doc! {"_id" : id})
        .await
    {
        Ok(Some(doc)) => Ok(doc),
        Ok(None) => Err(DbClassError::OtherError {
            err: "id not found ,please use other id".to_string(),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Some thing went wrong to item by id".to_string(),
        }),
    }
}
