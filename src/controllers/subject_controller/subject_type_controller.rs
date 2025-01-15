use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::subject_model::subject_model_model::{
        SubjectTypeModel, SubjectTypeModelGet, SubjectTypeModelNew, SubjectTypeModelPut,
    },
    AppState,
};

pub async fn create_subject_type(
    state: Arc<AppState>,
    subject_type: SubjectTypeModelNew,
) -> DbClassResult<SubjectTypeModelGet> {
    let create = state
        .db
        .subject_type
        .create(
            SubjectTypeModel::new(subject_type),
            Some("subject_type".to_string()),
        )
        .await?;
    let get = state
        .db
        .subject_type
        .get_one_by_id(create, Some("subject_type".to_string()))
        .await?;
    Ok(SubjectTypeModel::format(get))
}

pub async fn get_all_subject_type(state: Arc<AppState>) -> DbClassResult<Vec<SubjectTypeModelGet>> {
    let get = state
        .db
        .subject_type
        .get_many(None, Some("subject_type".to_string()))
        .await?;
    Ok(get.into_iter().map(SubjectTypeModel::format).collect())
}

pub async fn get_subject_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectTypeModelGet> {
    let get = state
        .db
        .subject_type
        .get_one_by_id(id, Some("subject_type".to_string()))
        .await?;
    Ok(SubjectTypeModel::format(get))
}

pub async fn update_subject_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    subject_type: SubjectTypeModelPut,
) -> DbClassResult<SubjectTypeModelGet> {
    let _ = state
        .db
        .subject_type
        .update(
            id,
            SubjectTypeModel::put(subject_type),
            Some("subject_type".to_string()),
        )
        .await;
    let get = get_subject_type_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_subject_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectTypeModelGet> {
    let delete = state
        .db
        .subject_type
        .delete(id, Some("subject_type".to_string()))
        .await?;
    Ok(SubjectTypeModel::format(delete))
}
