use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::class_model::class_room_model::{
        ClassRoomModel, ClassRoomModelGet, ClassRoomModelNew, ClassRoomModelPut,
    },
    AppState,
};

pub async fn create_class_room(
    state: Arc<AppState>,
    class_room: ClassRoomModelNew,
) -> DbClassResult<ClassRoomModelGet> {
    let create = state
        .db
        .class_room
        .create(
            ClassRoomModel::new(class_room),
            Some("class_room".to_string()),
        )
        .await?;
    let get = state
        .db
        .class_room
        .get_one_by_id(create, Some("class_room".to_string()))
        .await?;
    Ok(ClassRoomModel::format(get))
}

pub async fn get_all_class_room(state: Arc<AppState>) -> DbClassResult<Vec<ClassRoomModelGet>> {
    let get = state
        .db
        .class_room
        .get_many(None, Some("class_room".to_string()))
        .await?;
    Ok(get.into_iter().map(ClassRoomModel::format).collect())
}

pub async fn get_class_room_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassRoomModelGet> {
    let get = state
        .db
        .class_room
        .get_one_by_id(id, Some("class_room".to_string()))
        .await?;
    Ok(ClassRoomModel::format(get))
}

pub async fn update_class_room_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class_room: ClassRoomModelPut,
) -> DbClassResult<ClassRoomModelGet> {
    let _ = state
        .db
        .class_room
        .update(
            id,
            ClassRoomModel::put(class_room),
            Some("class_room".to_string()),
        )
        .await;
    let get = get_class_room_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_class_room_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassRoomModelGet> {
    let delete = state
        .db
        .class_room
        .delete(id, Some("class_room".to_string()))
        .await?;
    Ok(ClassRoomModel::format(delete))
}
