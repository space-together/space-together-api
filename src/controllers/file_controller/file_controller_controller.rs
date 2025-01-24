use mongodb::bson::oid::ObjectId;
use std::{str::FromStr, sync::Arc};

use crate::{
    error::db_class_error::DbClassResult,
    models::file_model::file_model_model::{FileModel, FileModelGet, FileModelNew, FileModelPut},
    AppState,
};

use super::file_type_controller::get_file_type_by_username;

pub async fn create_file(state: Arc<AppState>, file: FileModelNew) -> DbClassResult<FileModelGet> {
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

pub async fn update_file_image(
    state: Arc<AppState>,
    file: String,
    id: &str,
) -> DbClassResult<String> {
    let change_file_id = ObjectId::from_str(id).unwrap();
    let update_file_model = FileModelPut {
        src: Some(file),
        file_type: None,
        description: None,
    };
    let update_file = update_file_by_id(state.clone(), change_file_id, update_file_model).await?;
    Ok(update_file.id)
}

pub async fn handle_symbol_update(
    state: Arc<AppState>,
    file: String,
    existing_symbol_id: Option<String>,
) -> DbClassResult<String> {
    if let Some(file_id) = existing_symbol_id {
        update_file_image(state.clone(), file, &file_id).await
    } else {
        create_file_image(state.clone(), file, "Education symbol".to_string()).await
    }
}
