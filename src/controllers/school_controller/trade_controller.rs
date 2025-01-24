use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::file_controller::file_controller_controller::{
        create_file_image, get_file_by_id, handle_symbol_update,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{
        classes::db_crud::GetManyByField, functions::resources::check_if_exit::UsernameValidator,
    },
    models::school_model::trade_model::{TradeModel, TradeModelGet, TradeModelNew, TradeModelPut},
    AppState,
};

use super::sector_controller::get_sector_by_id;

async fn get_other_collection(
    state: Arc<AppState>,
    trade: TradeModel,
) -> DbClassResult<TradeModelGet> {
    let mut format_trade = TradeModel::format(trade.clone());
    if let Some(ref sector_id) = trade.sector_id {
        let get_sector = get_sector_by_id(state.clone(), *sector_id).await?;

        if let Some(sector_username) = get_sector.username {
            format_trade.sector = Some(sector_username);
        } else {
            format_trade.sector = Some(get_sector.name);
        }
    }

    if let Some(symbol_id) = trade.symbol_id {
        let get_symbol = get_file_by_id(state.clone(), symbol_id).await?;
        format_trade.symbol = Some(get_symbol.src);
    }
    Ok(format_trade)
}

pub async fn validate_trade_username(
    state: Arc<AppState>,
    username: &str,
    id_to_exclude: Option<ObjectId>,
) -> DbClassResult<()> {
    let validator = UsernameValidator::new(state.clone());

    validator
        .validate(username, id_to_exclude, move |state, username| {
            let username = username.to_string();
            Box::pin(async move {
                let trade = get_trade_by_username(state, username.clone()).await;
                trade.map(|trade| Some(trade.id)).or_else(|err| {
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

pub async fn create_trade(
    state: Arc<AppState>,
    mut trade: TradeModelNew,
) -> DbClassResult<TradeModelGet> {
    if let Some(ref username) = trade.username {
        let _ = validate_trade_username(state.clone(), username, None).await;
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

    if let Some(file) = trade.symbol {
        let symbol = create_file_image(state.clone(), file, "Trade symbol".to_string()).await?;
        trade.symbol = Some(symbol);
    }

    let index = IndexModel::builder()
        .keys(doc! {"username" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.trade.collection.create_index(index).await {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    let create = state
        .db
        .trade
        .create(TradeModel::new(trade), Some("trade".to_string()))
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
            err: format!("Trade not found by username [{}]", &username),
        })?;

    get_other_collection(state, get).await
}

pub async fn get_trade_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<TradeModelGet> {
    let get = state
        .db
        .trade
        .get_one_by_id(id, Some("trade".to_string()))
        .await?;

    get_other_collection(state, get).await
}

pub async fn get_trade_by_sector(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<Vec<TradeModelGet>> {
    let get = state
        .db
        .trade
        .get_many(
            Some(GetManyByField {
                field: "sector_id".to_string(),
                value: id,
            }),
            Some("Trades".to_string()),
        )
        .await?;
    let mut trades = Vec::new();
    for trade in get {
        trades.push(get_other_collection(state.clone(), trade).await?);
    }

    Ok(trades)
}

pub async fn get_all_trade(state: Arc<AppState>) -> DbClassResult<Vec<TradeModelGet>> {
    let get_all = state
        .db
        .trade
        .get_many(None, Some("trade".to_string()))
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
    mut trade: TradeModelPut,
) -> DbClassResult<TradeModelGet> {
    let exit_trade = get_trade_by_id(state.clone(), id).await?;
    if let Some(file) = trade.symbol {
        trade.symbol = Some(handle_symbol_update(state.clone(), file, exit_trade.symbol).await?);
    }

    let _ = state
        .db
        .trade
        .update(id, TradeModel::put(trade), Some("trade".to_string()))
        .await?;

    let get = state
        .db
        .trade
        .get_one_by_id(id, Some("trade".to_string()))
        .await?;

    Ok(TradeModel::format(get))
}

pub async fn delete_trade_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<TradeModelGet> {
    let get = get_trade_by_id(state.clone(), id).await?;
    let _ = state.db.trade.delete(id, Some("trade".to_string())).await?;
    Ok(get)
}
