use std::sync::Arc;

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
    libs::functions::characters_fn::is_valid_username,
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
    AppState,
};

pub async fn create_class(
    state: Arc<AppState>,
    class: ClassModelNew,
) -> DbClassResult<ClassModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"username" : 1,"code" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.trade.collection.create_index(index).await {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    if let Some(ref username) = class.username {
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_class_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username sector is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    }
    let create = state
        .db
        .class
        .create(ClassModel::new(class), Some("class".to_string()))
        .await?;
    let get = get_class_by_id(state, create).await?;
    Ok(get)
}

pub async fn get_class_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Sector not found by username [{}]", &username),
        })?;

    let mut trade_name: Option<String> = None;

    if let Some(ref trade_id) = get.trade_id {
        let get_trade = get_trade_by_id(state.clone(), *trade_id).await?;

        if let Some(trade_username) = get_trade.username {
            trade_name = Some(trade_username);
        } else {
            trade_name = Some(get_trade.name);
        }
    }

    let mut sector_name: Option<String> = None;

    if let Some(ref sector_id) = get.sector_id {
        let get_sector = get_sector_by_id(state.clone(), *sector_id).await?;

        if let Some(sector_username) = get_sector.username {
            sector_name = Some(sector_username);
        } else {
            sector_name = Some(get_sector.name);
        }
    }

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
    let _ = state
        .db
        .class
        .update(id, ClassModel::put(class), Some("class".to_string()))
        .await;
    let get = get_class_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassModelGet> {
    let delete = state.db.class.delete(id, Some("class".to_string())).await?;
    Ok(ClassModel::format(delete))
}
