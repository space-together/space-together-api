use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    models::class_model::class_room_type_model::{
        ClassRoomTypeModel, ClassRoomTypeModelGet, ClassRoomTypeModelNew, ClassRoomTypeModelPut,
    },
    AppState,
};

use super::class_room_controller::get_all_class_room_by_type;

pub async fn create_class_room_type(
    state: Arc<AppState>,
    class_room_type: ClassRoomTypeModelNew,
) -> DbClassResult<ClassRoomTypeModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"username": 1,})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .class
        .collection
        .create_index(index)
        .await
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

    if let Some(ref username) = class_room_type.username {
        if get_class_room_type_by_username(state.clone(), username.clone())
            .await
            .is_ok()
        {
            return Err(DbClassError::OtherError {
                err: format!(
                    "class room type username name is ready exit [{}] , please try other username",
                    username
                ),
            });
        }
    }

    let create = state
        .db
        .class_room_type
        .create(
            ClassRoomTypeModel::new(class_room_type),
            Some("class_room_type".to_string()),
        )
        .await?;
    let get = state
        .db
        .class_room_type
        .get_one_by_id(create, Some("class_room_type".to_string()))
        .await?;
    Ok(ClassRoomTypeModel::format(get))
}

pub async fn get_all_class_room_type(
    state: Arc<AppState>,
) -> DbClassResult<Vec<ClassRoomTypeModelGet>> {
    let get = state
        .db
        .class_room_type
        .get_many(None, Some("class_room_type".to_string()))
        .await?;
    Ok(get.into_iter().map(ClassRoomTypeModel::format).collect())
}

pub async fn get_class_room_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassRoomTypeModelGet> {
    let get = state
        .db
        .class_room_type
        .get_one_by_id(id, Some("class_room_type".to_string()))
        .await?;
    Ok(ClassRoomTypeModel::format(get))
}

pub async fn get_class_room_type_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<ClassRoomTypeModelGet> {
    let get = state
        .db
        .class_room_type
        .collection
        .find_one(doc! {"username": &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Class not found by username [{}]", &username),
        })?;
    Ok(ClassRoomTypeModel::format(get))
}

pub async fn update_class_room_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class_room_type: ClassRoomTypeModelPut,
) -> DbClassResult<ClassRoomTypeModelGet> {
    let _ = state
        .db
        .class_room_type
        .update(
            id,
            ClassRoomTypeModel::put(class_room_type),
            Some("class_room_type".to_string()),
        )
        .await;
    let get = get_class_room_type_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_class_room_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassRoomTypeModelGet> {
    let get_class_rooms = get_all_class_room_by_type(state.clone(), id).await;
    if let Ok(class_rooms) = get_class_rooms {
        if !class_rooms.is_empty() {
            return Err(DbClassError::OtherError { err: "You can not delete class room role because they are other document using it, delete those collection and try again".to_string() });
        }
    }

    let get = get_class_room_type_by_id(state.clone(), id).await?;
    let _ = state
        .db
        .class_room_type
        .delete(id, Some("class_room_type".to_string()))
        .await?;
    Ok(get)
}
