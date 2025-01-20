use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::school_controller::{
        sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::{
        characters_fn::is_valid_username,
        resources::check_if_exit::{check_sector_trade_exit, CheckSectorTradeExitModel},
    },
    models::class_model::class_room_model::{
        ClassRoomModel, ClassRoomModelGet, ClassRoomModelNew, ClassRoomModelPut,
    },
    AppState,
};

use super::class_room_type_controller::get_class_room_type_by_id;

pub async fn create_class_room(
    state: Arc<AppState>,
    class_room: ClassRoomModelNew,
) -> DbClassResult<ClassRoomModelGet> {
    if let Some(ref username) = class_room.username {
        is_valid_username(username).map_err(|err| DbClassError::OtherError {
            err: err.to_string(),
        })?;

        if get_class_room_by_username(state.clone(), username.clone())
            .await
            .is_ok()
        {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username sector already exists [{}], please try another",
                    username
                ),
            });
        }
    }

    if let Some(ref class_room_id) = class_room.class_room_type {
        let id = ObjectId::from_str(class_room_id).map_err(|_| DbClassError::OtherError {
            err: format!(
                "Class room type ID is invalid [{}], please try another",
                class_room_id
            ),
        })?;

        get_class_room_type_by_id(state.clone(), id)
            .await
            .map_err(|_| DbClassError::OtherError {
                err: format!(
                    "Class room type ID not found [{}], please try another",
                    class_room_id
                ),
            })?;
    }

    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class_room.sector.clone(),
            trade: class_room.trade.clone(),
        },
    )
    .await?;

    let index = IndexModel::builder()
        .keys(doc! {"username": 1, "code": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .class
        .collection
        .create_index(index)
        .await
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

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

    let trade_name = if let Some(ref trade_id) = get.trade_id {
        let trade = get_trade_by_id(state.clone(), *trade_id).await?;
        trade.username.or(Some(trade.name))
    } else {
        None
    };

    let sector_name = if let Some(ref sector_id) = get.sector_id {
        let sector = get_sector_by_id(state.clone(), *sector_id).await?;
        sector.username.or(Some(sector.name))
    } else {
        None
    };

    let class_room_type_name = if let Some(ref class_room_type_id) = get.class_room_type_id {
        let class_room_type = get_class_room_type_by_id(state.clone(), *class_room_type_id).await?;
        class_room_type.username.or(Some(class_room_type.name))
    } else {
        None
    };

    let mut class = ClassRoomModel::format(get);
    class.trade = trade_name;
    class.sector = sector_name;
    class.class_room_type = class_room_type_name;
    Ok(class)
}

pub async fn get_class_room_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<ClassRoomModelGet> {
    let get = state
        .db
        .class_room
        .collection
        .find_one(doc! {"username": &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Class not found by username [{}]", &username),
        })?;

    let trade_name = if let Some(ref trade_id) = get.trade_id {
        let trade = get_trade_by_id(state.clone(), *trade_id).await?;
        trade.username.or(Some(trade.name))
    } else {
        None
    };

    let sector_name = if let Some(ref sector_id) = get.sector_id {
        let sector = get_sector_by_id(state.clone(), *sector_id).await?;
        sector.username.or(Some(sector.name))
    } else {
        None
    };

    let mut class = ClassRoomModel::format(get);
    class.trade = trade_name;
    class.sector = sector_name;
    Ok(class)
}

pub async fn update_class_room_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class_room: ClassRoomModelPut,
) -> DbClassResult<ClassRoomModelGet> {
    if let Some(ref class_room_id) = class_room.class_room_type {
        let id = ObjectId::from_str(class_room_id).map_err(|_| DbClassError::OtherError {
            err: format!(
                "Class room type ID is invalid [{}], please try another",
                class_room_id
            ),
        })?;

        get_class_room_type_by_id(state.clone(), id)
            .await
            .map_err(|_| DbClassError::OtherError {
                err: format!(
                    "Class room type ID not found [{}], please try another",
                    class_room_id
                ),
            })?;
    }

    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class_room.sector.clone(),
            trade: class_room.trade.clone(),
        },
    )
    .await?;

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
