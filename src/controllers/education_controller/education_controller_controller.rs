use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::education_model::education_model_model::{
        EducationModel, EducationModelGet, EducationModelNew, EducationModelPut,
    },
    AppState,
};

pub async fn create_education(
    state: Arc<AppState>,
    education: EducationModelNew,
) -> DbClassResult<EducationModelGet> {
    let create = state
        .db
        .education
        .create(
            EducationModel::new(education),
            Some("education".to_string()),
        )
        .await?;
    let get = state
        .db
        .education
        .get_one_by_id(create, Some("education".to_string()))
        .await?;
    Ok(EducationModel::format(get))
}

pub async fn get_all_education(state: Arc<AppState>) -> DbClassResult<Vec<EducationModelGet>> {
    let get = state
        .db
        .education
        .get_many(None, Some("education".to_string()))
        .await?;
    Ok(get.into_iter().map(EducationModel::format).collect())
}

pub async fn get_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<EducationModelGet> {
    let get = state
        .db
        .education
        .get_one_by_id(id, Some("education".to_string()))
        .await?;
    Ok(EducationModel::format(get))
}

pub async fn update_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    education: EducationModelPut,
) -> DbClassResult<EducationModelGet> {
    let _ = state
        .db
        .education
        .update(
            id,
            EducationModel::put(education),
            Some("education".to_string()),
        )
        .await;
    let get = get_education_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<EducationModelGet> {
    let delete = state
        .db
        .education
        .delete(id, Some("education".to_string()))
        .await?;
    Ok(EducationModel::format(delete))
}
