use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    models::file_model::file_model_model::{FileModel, FileModelGet, FileModelNew, FileModelPut},
    AppState,
};

use super::file_type_controller::get_file_type_by_username;

pub async fn create_file(state: Arc<AppState>, file: FileModelNew) -> DbClassResult<FileModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"username": 1,})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .file
        .collection
        .create_index(index)
        .await
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

    let create = state
        .db
        .file
        .create(FileModel::new(file), Some("file".to_string()))
        .await?;

    get_file_by_id(state, create).await
}

pub async fn get_file_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<FileModelGet> {
    let get = state
        .db
        .file
        .get_one_by_id(id, Some("file".to_string()))
        .await?;
    Ok(FileModel::format(get))
}

pub async fn get_all_file(state: Arc<AppState>) -> DbClassResult<Vec<FileModelGet>> {
    let get = state
        .db
        .file
        .get_many(None, Some("file".to_string()))
        .await?;
    Ok(get.into_iter().map(FileModel::format).collect())
}

pub async fn update_file_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    file: FileModelPut,
) -> DbClassResult<FileModelGet> {
    state
        .db
        .file
        .update(id, FileModel::put(file), Some("file".to_string()))
        .await?;
    get_file_by_id(state, id).await
}

pub async fn delete_file_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<FileModelGet> {
    let delete = state.db.file.delete(id, Some("file".to_string())).await?;
    Ok(FileModel::format(delete))
}

pub async fn create_file_image(
    state: Arc<AppState>,
    file: String,
    description: String,
) -> DbClassResult<String> {
    let get_file_type = get_file_type_by_username(state.clone(), "image".to_string()).await?;
    let new_file = FileModelNew {
        src: file,
        description: Some(description),
        file_type: get_file_type.id,
    };

    let create_file = create_file(state.clone(), new_file).await?;

    Ok(create_file.id)
}

pub async fn get_file_image(state: Arc<AppState>, id: &String) -> DbClassResult<String> {
    let image_id = ObjectId::from_str(id).map_err(|_| DbClassError::OtherError {
        err: format!("Invalid Image id [{}]", id),
    })?;
    let get = get_file_by_id(state, image_id).await?;
    Ok(get.src)
}
