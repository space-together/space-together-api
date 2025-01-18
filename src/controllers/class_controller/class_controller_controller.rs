use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

use crate::{
    controllers::school_controller::{
        sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::characters_fn::{generate_code, generate_username, is_valid_username},
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
struct CheckOtherExitModel {
    username: Option<String>,
    sector: Option<String>,
    trade: Option<String>,
}

async fn check_other_exit(state: Arc<AppState>, exits: CheckOtherExitModel) -> DbClassResult<()> {
    if let Some(ref username) = exits.username {
        is_valid_username(username).map_err(|err| DbClassError::OtherError {
            err: err.to_string(),
        })?;

        if get_class_by_username(state.clone(), username.clone())
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

    if let Some(ref sector_id) = exits.sector {
        let id = ObjectId::from_str(sector_id).map_err(|_| DbClassError::OtherError {
            err: format!("Sector ID is invalid [{}], please try another", sector_id),
        })?;

        get_sector_by_id(state.clone(), id)
            .await
            .map_err(|_| DbClassError::OtherError {
                err: format!("Sector ID not found [{}], please try another", sector_id),
            })?;
    }

    if let Some(ref trade_id) = exits.trade {
        let id = ObjectId::from_str(trade_id).map_err(|_| DbClassError::OtherError {
            err: format!("Trade ID is invalid [{}], please try another", trade_id),
        })?;

        get_trade_by_id(state.clone(), id)
            .await
            .map_err(|_| DbClassError::OtherError {
                err: format!("Trade ID not found [{}], please try another", trade_id),
            })?;
    }

    Ok(())
}

pub async fn create_class(
    state: Arc<AppState>,
    mut class: ClassModelNew,
) -> DbClassResult<ClassModelGet> {
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

    check_other_exit(
        state.clone(),
        CheckOtherExitModel {
            username: class.username.clone(),
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;

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
    username: String,
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

    let mut class = ClassModel::format(get);
    class.trade = trade_name;
    class.sector = sector_name;
    Ok(class)
}

pub async fn get_class_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .get_one_by_id(id, Some("class".to_string()))
        .await?;
    Ok(ClassModel::format(get))
}

pub async fn get_all_class(state: Arc<AppState>) -> DbClassResult<Vec<ClassModelGet>> {
    let get = state
        .db
        .class
        .get_many(None, Some("class".to_string()))
        .await?;
    Ok(get.into_iter().map(ClassModel::format).collect())
}

pub async fn update_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class: ClassModelPut,
) -> DbClassResult<ClassModelGet> {
    check_other_exit(
        state.clone(),
        CheckOtherExitModel {
            username: class.username.clone(),
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;
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
    let delete = state.db.class.delete(id, Some("class".to_string())).await?;
    Ok(ClassModel::format(delete))
}
