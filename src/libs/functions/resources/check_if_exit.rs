use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    controllers::school_controller::{
        sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckSectorTradeExitModel {
    pub sector: Option<String>,
    pub trade: Option<String>,
}

pub async fn check_sector_trade_exit(
    state: Arc<AppState>,
    exits: CheckSectorTradeExitModel,
) -> DbClassResult<()> {
    if let Some(ref sector_id) = exits.sector {
        if !sector_id.is_empty() {
            let id = ObjectId::from_str(sector_id).map_err(|_| DbClassError::OtherError {
                err: format!("Sector ID is invalid [{}], please try another", sector_id),
            })?;

            get_sector_by_id(state.clone(), id)
                .await
                .map_err(|_| DbClassError::OtherError {
                    err: format!("Sector ID not found [{}], please try another", sector_id),
                })?;
        }
    }

    if let Some(ref trade_id) = exits.trade {
        if !trade_id.is_empty() {
            let id = ObjectId::from_str(trade_id).map_err(|_| DbClassError::OtherError {
                err: format!("Trade ID is invalid [{}], please try another", trade_id),
            })?;

            get_trade_by_id(state.clone(), id)
                .await
                .map_err(|_| DbClassError::OtherError {
                    err: format!("Trade ID not found [{}], please try another", trade_id),
                })?;
        }
    }

    Ok(())
}
