use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::subject_model::subject_model_model::{
        SubjectModel, SubjectModelGet, SubjectModelNew, SubjectModelPut,
    },
    AppState,
};

pub async fn create_subject(
    state: Arc<AppState>,
    subject: SubjectModelNew,
) -> DbClassResult<SubjectModelGet> {
    let create = state
        .db
        .subject
        .create(SubjectModel::new(subject), Some("subject".to_string()))
        .await?;
    let get = state
        .db
        .subject
        .get_one_by_id(create, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(get))
}

pub async fn get_all_subject(state: Arc<AppState>) -> DbClassResult<Vec<SubjectModelGet>> {
    let get = state
        .db
        .subject
        .get_many(None, Some("subject".to_string()))
        .await?;
    Ok(get.into_iter().map(SubjectModel::format).collect())
}

pub async fn get_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectModelGet> {
    let get = state
        .db
        .subject
        .get_one_by_id(id, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(get))
}

pub async fn update_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    subject: SubjectModelPut,
) -> DbClassResult<SubjectModelGet> {
    let _ = state
        .db
        .subject
        .update(id, SubjectModel::put(subject), Some("subject".to_string()))
        .await;
    let get = get_subject_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectModelGet> {
    let delete = state
        .db
        .subject
        .delete(id, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(delete))
}
