use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::file_controller::file_controller_controller::{create_file_image, get_file_by_id}, 
    error::db_class_error::{DbClassError, DbClassResult}, 
    libs::{classes::db_crud::GetManyByField, functions::characters_fn::is_valid_username}, 
    models::education_model::education_model_model::{
        EducationModel, EducationModelGet, EducationModelNew, EducationModelPut,
    }, 
    AppState
};
use futures::future::join_all;

async fn validate_username(
    state: &Arc<AppState>,
    username: &str,
    id_to_exclude: Option<&ObjectId>,
) -> DbClassResult<()> {
    is_valid_username(username).map_err(|err| DbClassError::OtherError {
        err: err.to_string(),
    })?;

    if let Ok(existing_education) = get_education_by_username(state.clone(), username.to_string()).await {
        if id_to_exclude.is_none_or(|exclude_id| existing_education.id != exclude_id.to_string()) {
            return Err(DbClassError::OtherError {
                err: format!("Username '{}' is already in use, please try another.", username),
            });
        }
    }

    Ok(())
}

async fn get_other_collections(
    state: &Arc<AppState>,
    item: EducationModel,
) -> DbClassResult<EducationModelGet> {
    let mut education = EducationModel::format(item.clone());

    if let Some(symbol_id) = item.symbol {
        if let Ok(id) = ObjectId::from_str(&symbol_id) {
            if let Ok(image) = get_file_by_id(state.clone(), id).await {
                education.symbol = Some(image.src);
            }
        }
    }
    Ok(education)
}

pub async fn create_education(
    state: Arc<AppState>,
    mut education: EducationModelNew,
) -> DbClassResult<EducationModelGet> {
    let index = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state.db.education.collection.create_index(index).await.ok();

    let username = education.username.as_ref()
        .ok_or_else(|| DbClassError::OtherError { err: "Username is missing".to_string() })?;

    validate_username(&state, username, None).await?;

    if let Some(file) = education.symbol.take() {
        education.symbol = Some(create_file_image(state.clone(), file, "Education symbol".to_string()).await?);
    }

    let created_id = state.db.education.create(EducationModel::new(education), Some("education".to_string())).await?;
    get_education_by_id(state, created_id).await
}

pub async fn get_all_education(state: Arc<AppState>) -> DbClassResult<Vec<EducationModelGet>> {
    let educations = state.db.education.get_many(None, Some("education".to_string())).await?;
    let results = join_all(educations.into_iter().map(|edu| get_other_collections(&state, edu))).await;
    
    Ok(results.into_iter().filter_map(Result::ok).collect())
}
pub async fn get_education_by_id(
    state: Arc<AppState>, 
    id: ObjectId
) -> DbClassResult<EducationModelGet> {
    let education = state.db.education.get_one_by_id(id, Some("education".to_string())).await?;

    get_other_collections(&state, education).await
}


pub async fn get_education_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<EducationModelGet> {
    let get = state
        .db
        .education
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Education not found by username [{}]", &username),
        })?;
Ok(EducationModel::format(get))
}



pub async fn update_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    education: EducationModelPut,
) -> DbClassResult<EducationModelGet> {
    if let Some(username) = &education.username {
        validate_username(&state, username, Some(&id)).await?;
    }

    state.db.education.update(id, EducationModel::put(education), Some("education".to_string())).await?;
    get_education_by_id(state, id).await
}

pub async fn delete_education_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<EducationModelGet> {
    if let Ok(sectors) = state.db.sector.get_many(
        Some(GetManyByField { field: "education_id".to_string(), value: id }),
        Some("education".to_string()),
    ).await {
        if !sectors.is_empty() {
            return Err(DbClassError::OtherError { 
                err: "Cannot delete education account because associated sectors exist. Delete those first.".to_string() 
            });
        }
    }

    let education = get_education_by_id(state.clone(), id).await?;
    state.db.education.delete(id, Some("education".to_string())).await?;
    Ok(education)
}
