use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};
use std::sync::Arc;

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::characters_fn::{generate_username, is_valid_username},
    models::file_model::file_type_model::{
        FileTypeModel, FileTypeModelGet, FileTypeModelNew, FileTypeModelPut,
    },
    AppState,
};

pub async fn create_file_type(
    state: Arc<AppState>,
    mut file_type: FileTypeModelNew,
) -> DbClassResult<FileTypeModelGet> {
    if let Some(ref username) = file_type.username {
        let check_username = is_valid_username(username);
        if let Err(e) = check_username {
            return Err(DbClassError::OtherError { err: e });
        }
        let get_username = get_file_type_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "File type username is leady exit [{}], please type other username",
                    username
                ),
            });
        }
    }

    let index = IndexModel::builder()
        .keys(doc! {"username": 1,})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .file_type
        .collection
        .create_index(index)
        .await
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

    if file_type.username.is_none() {
        file_type.username = Some(generate_username(&file_type.name));
    }
    let create = state
        .db
        .file_type
        .create(FileTypeModel::new(file_type), Some("file_type".to_string()))
        .await?;

    get_file_type_by_id(state, create).await
}

pub async fn get_file_type_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<FileTypeModelGet> {
    let get = state
        .db
        .file_type
        .collection
        .find_one(doc! {"username": &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("file_type not found by username [{}]", &username),
        })?;

    let file_type = FileTypeModel::format(get);
    Ok(file_type)
}

pub async fn get_file_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<FileTypeModelGet> {
    let get = state
        .db
        .file_type
        .get_one_by_id(id, Some("file_type".to_string()))
        .await?;
    Ok(FileTypeModel::format(get))
}

pub async fn get_all_file_type(state: Arc<AppState>) -> DbClassResult<Vec<FileTypeModelGet>> {
    let get = state
        .db
        .file_type
        .get_many(None, Some("file_type".to_string()))
        .await?;
    Ok(get.into_iter().map(FileTypeModel::format).collect())
}

pub async fn update_file_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    file_type: FileTypeModelPut,
) -> DbClassResult<FileTypeModelGet> {
    state
        .db
        .file_type
        .update(
            id,
            FileTypeModel::put(file_type),
            Some("file_type".to_string()),
        )
        .await?;
    get_file_type_by_id(state, id).await
}

pub async fn delete_file_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<FileTypeModelGet> {
    let delete = state
        .db
        .file_type
        .delete(id, Some("file_type".to_string()))
        .await?;
    Ok(FileTypeModel::format(delete))
}
