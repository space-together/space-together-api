use std::future::Future;
use std::pin::Pin;
use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::libs::functions::characters_fn::is_valid_username;
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

///////////////////////////////////////////////////////////////////////////////////////
pub struct UsernameValidator {
    state: Arc<AppState>,
}

impl UsernameValidator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Validate the uniqueness and format of a username across a given collection.
    /// Dynamically calls the appropriate `get_username_fn` to check the collection.
    pub async fn validate<F>(
        &self,
        username: &str,
        id_to_exclude: Option<ObjectId>,
        get_username_fn: F,
    ) -> DbClassResult<()>
    where
        F: Fn(
            Arc<AppState>,
            &str,
        ) -> Pin<Box<dyn Future<Output = DbClassResult<Option<String>>> + Send>>,
    {
        // Check if the username format is valid
        is_valid_username(username).map_err(|err| DbClassError::OtherError {
            err: err.to_string(),
        })?;

        // Check if the username already exists
        if let Some(existing_id) = get_username_fn(self.state.clone(), username).await? {
            if let Some(exclude_id) = id_to_exclude {
                if existing_id != exclude_id.to_string() {
                    return Err(DbClassError::OtherError {
                        err: format!("Username already exists [{}], please try another", username),
                    });
                }
            } else {
                return Err(DbClassError::OtherError {
                    err: format!("Username already exists [{}], please try another", username),
                });
            }
        }

        Ok(())
    }
}
