use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::{
        file_controller::file_controller_controller::{
            create_file_image, get_file_by_id, handle_symbol_update,
        },
        school_controller::{
            sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
        },
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{
        classes::db_crud::GetManyByField, functions::resources::check_if_exit::UsernameValidator,
    },
    models::class_model::class_room_model::{
        ClassRoomModel, ClassRoomModelGet, ClassRoomModelNew, ClassRoomModelPut,
    },
    AppState,
};

use super::class_room_type_controller::get_class_room_type_by_id;

async fn get_other_collection(
    state: Arc<AppState>,
    class_room: ClassRoomModel,
) -> DbClassResult<ClassRoomModelGet> {
    let mut format_class_room = ClassRoomModel::format(class_room.clone());

    if let Some(ref trade_id) = class_room.trade_id {
        let trade = get_trade_by_id(state.clone(), *trade_id).await?;
        format_class_room.trade = trade.username.or(Some(trade.name))
    }

    if let Some(ref sector_id) = class_room.sector_id {
        let sector = get_sector_by_id(state.clone(), *sector_id).await?;
        format_class_room.sector = sector.username.or(Some(sector.name))
    }

    if let Some(ref class_room_type_id) = class_room.class_room_type_id {
        let class_room_type = get_class_room_type_by_id(state.clone(), *class_room_type_id).await?;
        format_class_room.class_room_type = class_room_type.username.or(Some(class_room_type.name));
    };

    if let Some(symbol_id) = class_room.symbol_id {
        let get_symbol = get_file_by_id(state.clone(), symbol_id).await?;
        format_class_room.symbol = Some(get_symbol.src);
    };

    Ok(format_class_room)
}

pub async fn validate_class_room_username(
    state: Arc<AppState>,
    username: &str,
    id_to_exclude: Option<ObjectId>,
) -> DbClassResult<()> {
    let validator = UsernameValidator::new(state.clone());

    validator
        .validate(username, id_to_exclude, move |state, username| {
            let username = username.to_string();
            Box::pin(async move {
                let class_room = get_class_room_by_username(state, username.clone()).await;
                class_room.map(|trade| Some(trade.id)).or_else(|err| {
                    if matches!(err, DbClassError::OtherError { .. }) {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
            })
        })
        .await
}

pub async fn create_class_room(
    state: Arc<AppState>,
    mut class_room: ClassRoomModelNew,
) -> DbClassResult<ClassRoomModelGet> {
    if let Some(ref username) = class_room.username {
        let _ = validate_class_room_username(state.clone(), username, None).await;
    } else {
        return Err(DbClassError::OtherError {
            err: "Username is missing".to_string(),
        });
    }

    // Validate class room type
    if let Some(ref class_room_id) = class_room.class_room_type {
        if !class_room_id.is_empty() {
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
        } else {
            class_room.class_room_type = None
        }
    }

    // Validate trade
    if let Some(ref trade_id) = class_room.trade {
        if !trade_id.is_empty() {
            let id = ObjectId::from_str(trade_id).map_err(|_| DbClassError::OtherError {
                err: format!("Trade ID is invalid [{}], please try another", trade_id),
            })?;

            let get_by_trade = get_all_class_room_by_trade(state.clone(), id).await?;
            let trade =
                get_trade_by_id(state.clone(), id)
                    .await
                    .map_err(|_| DbClassError::OtherError {
                        err: format!("Trade ID not found [{}], please try another", trade_id),
                    })?;

            if let Some(num_class_room) = trade.class_rooms {
                if get_by_trade.len() >= num_class_room as usize {
                    return Err(DbClassError::OtherError {
                        err: format!(
                            "You cannot add another classroom in [{}] because the maximum limit of [{}] classrooms has been reached. The class is full",
                            trade.name, num_class_room
                        ),
                    });
                }
            }
        } else {
            class_room.trade = None
        }
    }

    // Validate sector
    if let Some(ref sector_id) = class_room.sector {
        if !sector_id.is_empty() {
            let id = ObjectId::from_str(sector_id).map_err(|_| DbClassError::OtherError {
                err: format!("Sector ID is invalid [{}], please try another", sector_id),
            })?;

            get_sector_by_id(state.clone(), id)
                .await
                .map_err(|_| DbClassError::OtherError {
                    err: format!("Sector ID not found [{}], please try another", sector_id),
                })?;
        } else {
            class_room.sector = None
        }
    }

    // Create a unique index for username and code
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

    if let Some(file) = class_room.symbol {
        let symbol =
            create_file_image(state.clone(), file, "class room symbol".to_string()).await?;
        class_room.symbol = Some(symbol);
    }
    // Create and retrieve the new classroom
    let create = state
        .db
        .class_room
        .create(
            ClassRoomModel::new(class_room),
            Some("class_room".to_string()),
        )
        .await?;
    let get = get_class_room_by_id(state, create).await?;
    Ok(get)
}

pub async fn get_all_class_room(state: Arc<AppState>) -> DbClassResult<Vec<ClassRoomModelGet>> {
    let get = state
        .db
        .class_room
        .get_many(None, Some("class_room".to_string()))
        .await?;
    let mut class_rooms: Vec<ClassRoomModelGet> = Vec::new();

    for class_room in get {
        let change = get_other_collection(state.clone(), class_room).await?;
        class_rooms.push(change);
    }

    Ok(class_rooms)
}

pub async fn get_all_class_room_by_trade(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<Vec<ClassRoomModelGet>> {
    let get = state
        .db
        .class_room
        .get_many(
            Some(GetManyByField {
                field: "trade_id".to_string(),
                value: id,
            }),
            Some("class_room".to_string()),
        )
        .await?;
    let mut class_rooms: Vec<ClassRoomModelGet> = Vec::new();

    for class_room in get {
        let change = get_other_collection(state.clone(), class_room).await?;
        class_rooms.push(change);
    }

    Ok(class_rooms)
}

pub async fn get_all_class_room_by_sector(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<Vec<ClassRoomModelGet>> {
    let get = state
        .db
        .class_room
        .get_many(
            Some(GetManyByField {
                field: "sector_id".to_string(),
                value: id,
            }),
            Some("class_room".to_string()),
        )
        .await?;
    let mut class_rooms: Vec<ClassRoomModelGet> = Vec::new();

    for class_room in get {
        let change = get_other_collection(state.clone(), class_room).await?;
        class_rooms.push(change);
    }

    Ok(class_rooms)
}

pub async fn get_all_class_room_by_type(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<Vec<ClassRoomModelGet>> {
    let get = state
        .db
        .class_room
        .get_many(
            Some(GetManyByField {
                field: "class_room_type_id".to_string(),
                value: id,
            }),
            Some("class_room".to_string()),
        )
        .await?;

    let mut class_rooms: Vec<ClassRoomModelGet> = Vec::new();

    for class_room in get {
        let change = get_other_collection(state.clone(), class_room).await?;
        class_rooms.push(change);
    }

    Ok(class_rooms)
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

    get_other_collection(state, get).await
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

    get_other_collection(state, get).await
}

pub async fn update_class_room_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    mut class_room: ClassRoomModelPut,
) -> DbClassResult<ClassRoomModelGet> {
    if let Some(username) = &class_room.username {
        validate_class_room_username(state.clone(), username, Some(id)).await?;
    }
    let existing_class_room = state
        .db
        .class_room
        .get_one_by_id(id, Some("class_room".to_string()))
        .await?;
    if let Some(file) = class_room.symbol {
        class_room.symbol =
            Some(handle_symbol_update(state.clone(), file, existing_class_room.symbol_id).await?);
    }

    if let Some(ref class_room_type_id) = class_room.class_room_type {
        if !class_room_type_id.is_empty() {
            let id =
                ObjectId::from_str(class_room_type_id).map_err(|_| DbClassError::OtherError {
                    err: format!(
                        "Class room type ID is invalid [{}], please try another",
                        class_room_type_id
                    ),
                })?;

            get_class_room_type_by_id(state.clone(), id)
                .await
                .map_err(|_| DbClassError::OtherError {
                    err: format!(
                        "Class room type ID not found [{}], please try another",
                        class_room_type_id
                    ),
                })?;
        } else {
            class_room.class_room_type = None
        }
    }

    if let Some(ref trade_id) = class_room.trade {
        if !trade_id.is_empty() {
            let id = ObjectId::from_str(trade_id).map_err(|_| DbClassError::OtherError {
                err: format!("Trade ID is invalid [{}], please try another", trade_id),
            })?;

            let get_by_trade = get_all_class_room_by_trade(state.clone(), id).await?;
            let trade =
                get_trade_by_id(state.clone(), id)
                    .await
                    .map_err(|_| DbClassError::OtherError {
                        err: format!("Trade ID not found [{}], please try another", trade_id),
                    })?;

            if let Some(num_class_room) = trade.class_rooms {
                if get_by_trade.len() >= num_class_room as usize {
                    return Err(DbClassError::OtherError { err: format!("You cannot add another classroom in [{}] because the maximum limit of [{}] classrooms has been reached. The class is full",trade.name , num_class_room) });
                }
            }
        } else {
            class_room.trade = None
        }
    }

    if let Some(ref sector_id) = class_room.sector {
        if !sector_id.is_empty() {
            let id = ObjectId::from_str(sector_id).map_err(|_| DbClassError::OtherError {
                err: format!("Sector ID is invalid [{}], please try another", sector_id),
            })?;

            get_sector_by_id(state.clone(), id)
                .await
                .map_err(|_| DbClassError::OtherError {
                    err: format!("Sector ID not found [{}], please try another", sector_id),
                })?;
        } else {
            class_room.sector = None
        }
    }

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
