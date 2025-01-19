use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::characters_fn::is_valid_username,
    models::school_model::trade_model::{TradeModel, TradeModelGet, TradeModelNew, TradeModelPut},
    AppState,
};

use super::sector_controller::get_sector_by_id;

pub async fn create_trade(
    state: Arc<AppState>,
    trade: TradeModelNew,
) -> DbClassResult<TradeModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"username" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.trade.collection.create_index(index).await {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    if let Some(ref username) = trade.username {
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_trade_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username trade is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    } else {
        return Err(DbClassError::OtherError {
            err: "Username is missing".to_string(),
        });
    }

    if let Some(ref sector) = trade.sector {
        let id = match ObjectId::from_str(sector) {
            Err(_) => {
                return Err(DbClassError::OtherError {
                    err: format!("Invalid trade id [{}], please try other id", sector),
                })
            }
            Ok(i) => i,
        };

        let get_sector = get_sector_by_id(state.clone(), id).await;

        if get_sector.is_err() {
            return Err(DbClassError::OtherError {
                err: format!("Sector id is not found [{}], please try other id", sector),
            });
        }
    }

    let create = state
        .db
        .trade
        .create(TradeModel::new(trade), Some("School section".to_string()))
        .await?;
    let get = get_trade_by_id(state, create).await?;
    Ok(get)
}

pub async fn get_trade_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<TradeModelGet> {
    let get = state
        .db
        .trade
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Sector not found by username [{}]", &username),
        })?;

    let mut sector_name: Option<String> = None;

    if let Some(ref education_id) = get.sector_id {
        let get_education = get_sector_by_id(state.clone(), *education_id).await?;

        if let Some(education_username) = get_education.username {
            sector_name = Some(education_username);
        } else {
            sector_name = Some(get_education.name);
        }
    }
    let mut sector = TradeModel::format(get);
    sector.sector = sector_name;
    Ok(sector)
}

pub async fn get_trade_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<TradeModelGet> {
    let get = state
        .db
        .trade
        .get_one_by_id(id, Some("School section".to_string()))
        .await?;

    let mut sector_name: Option<String> = None;

    if let Some(ref sector_id) = get.sector_id {
        let get_sector = get_sector_by_id(state.clone(), *sector_id).await?;

        if let Some(sector_username) = get_sector.username {
            sector_name = Some(sector_username);
        } else {
            sector_name = Some(get_sector.name);
        }
    }
    let mut sector = TradeModel::format(get);
    sector.sector = sector_name;
    Ok(sector)
}

pub async fn get_all_trade(state: Arc<AppState>) -> DbClassResult<Vec<TradeModelGet>> {
    let get_all = state
        .db
        .trade
        .get_many(None, Some("School section".to_string()))
        .await?;
    let mut trades: Vec<TradeModelGet> = Vec::new();

    for trade in get_all {
        let mut sector_name: Option<String> = None;
        let mut format_trade = TradeModel::format(trade.clone());

        if let Some(ref sector_id) = trade.sector_id {
            let get_sector = get_sector_by_id(state.clone(), *sector_id).await?;

            if let Some(sector_username) = get_sector.username {
                sector_name = Some(sector_username);
            } else {
                sector_name = Some(get_sector.name);
            }
        }
        format_trade.sector = sector_name;
        trades.push(format_trade);
    }

    Ok(trades)
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
    let _ = state
        .db
        .trade
        .delete(id, Some("School Section".to_string()))
        .await?;
    let get = get_trade_by_id(state, id).await?;
    Ok(get)
}
