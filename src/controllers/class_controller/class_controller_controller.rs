use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::{
        school_controller::{
            sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
        },
        user_controller::user_controller_controller::{
            controller_get_user_by_id, controller_user_get_user_by_email,
        },
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::{
        characters_fn::{generate_code, generate_username, is_valid_email, is_valid_username},
        resources::check_if_exit::{check_sector_trade_exit, CheckSectorTradeExitModel},
    },
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
    AppState,
};

use super::{
    class_room_controller::get_class_room_by_id, class_type_controller::get_class_type_by_id,
};

async fn get_other_collection(
    state: Arc<AppState>,
    class: ClassModel,
) -> DbClassResult<ClassModelGet> {
    let trade_name = if let Some(ref trade_id) = class.trade_id {
        let trade = get_trade_by_id(state.clone(), *trade_id).await?;
        trade.username.or(Some(trade.name))
    } else {
        None
    };

    let sector_name = if let Some(ref sector_id) = class.sector_id {
        let sector = get_sector_by_id(state.clone(), *sector_id).await?;
        sector.username.or(Some(sector.name))
    } else {
        None
    };

    let class_type = if let Some(ref class_type_id) = class.class_type_id {
        let document = get_class_type_by_id(state.clone(), *class_type_id).await?;
        document.username.or(Some(document.name))
    } else {
        None
    };

    let class_teacher = if let Some(ref class_teacher_id) = class.class_teacher_id {
        let document = controller_get_user_by_id(state.clone(), *class_teacher_id)
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;
        document.username.or(Some(document.name))
    } else {
        None
    };

    let class_room = if let Some(class_room_id) = class.class_room_id {
        let document = get_class_room_by_id(state.clone(), class_room_id)
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;
        document.username.or(Some(document.name))
    } else {
        None
    };

    let mut class = ClassModel::format(class);
    class.trade = trade_name;
    class.sector = sector_name;
    class.class_type = class_type;
    class.class_teacher = class_teacher;
    class.class_room = class_room;

    Ok(class)
}

pub async fn create_class(
    state: Arc<AppState>,
    mut class: ClassModelNew,
) -> DbClassResult<ClassModelGet> {
    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;

    if let Some(ref user_email) = class.class_teacher {
        is_valid_email(user_email).map_err(|e| DbClassError::OtherError { err: e })?;

        let teacher = controller_user_get_user_by_email(state.clone(), user_email.clone())
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

        if let Some(teacher_role) = teacher.role {
            if teacher_role != "Student" {
                class.class_teacher = Some(teacher.id);
            } else {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "This user is not allowed [{}] to create class, because his/her role is student [{}]",
                        teacher.name, teacher_role
                    ),
                });
            }
        } else {
            return Err(DbClassError::OtherError {
                err: format!(
                    "This user is not allowed [{}] to create class, because his/he don't have role",
                    teacher.name,
                ),
            });
        }
    }

    if let Some(ref username) = class.username {
        is_valid_username(username).map_err(|err| DbClassError::OtherError { err })?;
        let get_username = get_class_by_username(state.clone(), username).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username sector already exists [{}], please try another",
                    username
                ),
            });
        }
    }

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

    if class.username.is_none() {
        class.username = Some(generate_username(&class.name));
    }

    if class.code.is_none() {
        class.code = Some(generate_code());
    }

    let create = state
        .db
        .class
        .create(ClassModel::new(class), Some("class".to_string()))
        .await?;

    get_class_by_id(state, create).await
}

pub async fn get_class_by_username(
    state: Arc<AppState>,
    username: &String,
) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .collection
        .find_one(doc! {"username": &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Class not found by username [{}]", &username),
        })?;

    get_other_collection(state, get).await
}

pub async fn get_class_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .get_one_by_id(id, Some("class".to_string()))
        .await?;

    get_other_collection(state, get).await
}

pub async fn get_all_class(state: Arc<AppState>) -> DbClassResult<Vec<ClassModelGet>> {
    let get = state
        .db
        .class
        .get_many(None, Some("class".to_string()))
        .await?;

    let mut class_gets = Vec::new();

    for class in get {
        let my_class = get_other_collection(state.clone(), class).await?;
        class_gets.push(my_class);
    }

    Ok(class_gets)
}

pub async fn update_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    mut class: ClassModelPut,
) -> DbClassResult<ClassModelGet> {
    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;

    if let Some(ref user_email) = class.class_teacher {
        is_valid_email(user_email).map_err(|e| DbClassError::OtherError { err: e })?;

        let teacher = controller_user_get_user_by_email(state.clone(), user_email.clone())
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

        if let Some(teacher_role) = teacher.role {
            if teacher_role != "Student" {
                class.class_teacher = Some(teacher.id);
            } else {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "This user is not allowed [{}] to create class, because his/her role is student [{}]",
                        teacher.name, teacher_role
                    ),
                });
            }
        } else {
            return Err(DbClassError::OtherError {
                err: format!(
                    "This user is not allowed [{}] to create class, because his/he don't have role",
                    teacher.name,
                ),
            });
        }
    }

    if let Some(ref username) = class.username {
        is_valid_username(username).map_err(|err| DbClassError::OtherError { err })?;
        let get_username = get_class_by_username(state.clone(), username).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username sector already exists [{}], please try another",
                    username
                ),
            });
        }
    }

    state
        .db
        .class
        .update(id, ClassModel::put(class), Some("class".to_string()))
        .await?;
    get_class_by_id(state, id).await
}

pub async fn delete_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassModelGet> {
    let get = get_class_by_id(state.clone(), id).await?;
    state.db.class.delete(id, Some("class".to_string())).await?;
    Ok(get)
}
