use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    models::school_model::trade_model::{TradeModel, TradeModelGet, TradeModelNew, TradeModelPut},
    AppState,
};

pub async fn create_trade(
    state: Arc<AppState>,
    section: TradeModelNew,
) -> DbClassResult<TradeModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"name" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.trade.collection.create_index(index).await {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    let get = state
        .db
        .trade
        .collection
        .find_one(doc! {"name" : section.name.clone()})
        .await
        .map_err(|e| DbClassError::OtherError {
            err: format!(
                "Some thing went wrong to get school section error is : 😡 [{}] 😡",
                e
            ),
        })?;

    if let Some(r) = get {
        return Err(DbClassError::OtherError {
            err: format!(
                "School Section name already exists [{}], try other name",
                r.name
            ),
        });
    }

    let create = state
        .db
        .trade
        .create(TradeModel::new(section), Some("School section".to_string()))
        .await?;
    let get = state
        .db
        .trade
        .get_one_by_id(create, Some("School section".to_string()))
        .await?;
    Ok(TradeModel::format(get))
}

pub async fn get_all_trade(state: Arc<AppState>) -> DbClassResult<Vec<TradeModelGet>> {
    let get_all = state
        .db
        .trade
        .get_many(None, Some("School section".to_string()))
        .await?;
    Ok(get_all.into_iter().map(TradeModel::format).collect())
}

pub async fn get_trade_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<TradeModelGet> {
    let get = state
        .db
        .trade
        .get_one_by_id(id, Some("School section".to_string()))
        .await?;

    Ok(TradeModel::format(get))
}

pub async fn update_trade_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    section: TradeModelPut,
) -> DbClassResult<TradeModelGet> {
    let _ = state
        .db
        .trade
        .update(
            id,
            TradeModel::put(section),
            Some("School Section".to_string()),
        )
        .await?;

    let get = state
        .db
        .trade
        .get_one_by_id(id, Some("School section".to_string()))
        .await?;

    Ok(TradeModel::format(get))
}

pub async fn delete_trade_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<TradeModelGet> {
    if (state
        .db
        .class
        .class
        .find(doc! {"sections" : {"$in" : [id]}})
        .await)
        .is_ok()
    {
        return Err(DbClassError::OtherError {
            err: "You can not delete section bcs they are class using it".to_string(),
        });
    }
    let delete = state
        .db
        .trade
        .delete(id, Some("School Section".to_string()))
        .await?;
    Ok(TradeModel::format(delete))
}
