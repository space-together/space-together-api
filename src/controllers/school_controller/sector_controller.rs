use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::{education_controller::education_controller_controller::get_education_by_id, file_controller::file_controller_controller::{create_file_image, get_file_by_id, handle_symbol_update}},
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{classes::db_crud::GetManyByField, functions::resources::check_if_exit::UsernameValidator},
    models::school_model::sector_model::{
        SectorModel, SectorModelGet, SectorModelNew, SectorModelPut,
    },
    AppState,
};

use super::trade_controller::get_trade_by_sector;

pub async fn validate_sector_username(
    state: Arc<AppState>,
    username: &str,
    id_to_exclude: Option<ObjectId>,
) -> DbClassResult<()> {
    let validator = UsernameValidator::new(state.clone());

    validator
        .validate(username, id_to_exclude, move |state, username| {
            let username = username.to_string();
            Box::pin(async move {
                let sector = get_sector_by_username(state, username.clone()).await;
                sector.map(|sector| Some(sector.id)).or_else(|err| {
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

async fn get_other_collection (state: Arc<AppState> , sector: SectorModel) -> DbClassResult<SectorModelGet> {
    let mut format_sector = SectorModel::format(sector.clone());
    if let Some(ref education_id) = sector.education_id {
        let get_education = get_education_by_id(state.clone(), *education_id).await?;

        if let Some(education_username) = get_education.username {
            format_sector.education = Some(education_username);
        } else {
            format_sector.education = Some(get_education.name);
        }
    }

    if let Some(symbol_id) = sector.symbol_id {
        let get_symbol = get_file_by_id(state.clone(), symbol_id).await?;
        format_sector.symbol = Some(get_symbol.src);
    }
    Ok(format_sector)
}

pub async fn create_sector(
    state: Arc<AppState>,
   mut sector: SectorModelNew,
) -> DbClassResult<SectorModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {
        "username" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(err) = state.db.education.collection.create_index(index).await {
        return Err(DbClassError::OtherError {
            err: format!(
                "Can't create education bcs username is leady exit [{}] ",
                err
            ),
        });
    }

   if let Some(ref username) = sector.username {
   let _ = validate_sector_username(state.clone(), username, None).await;
   }

   if let Some(file) = sector.symbol {
    let symbol =   create_file_image(state.clone(), file, "Education symbol".to_string()).await?;
    sector.symbol = Some(symbol);
  }


    if let Some(ref education) = sector.education {
        let id = match ObjectId::from_str(education) {
            Err(_) => {
                return Err(DbClassError::OtherError {
                    err: format!("Invalid education id [{}], please try other id", education),
                })
            }
            Ok(i) => i,
        };

        let get_education = get_education_by_id(state.clone(), id).await;

        if get_education.is_err() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Education id is not found [{}], please try other id",
                    education
                ),
            });
        }
    }

    let create = state
        .db
        .sector
        .create(SectorModel::new(sector), Some("sector".to_string()))
        .await?;
    let get = get_sector_by_id(state.clone(), create).await?;

    Ok(get)
}

pub async fn get_sector_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<SectorModelGet> {
    let get = state
        .db
        .sector
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Sector not found by username [{}]", &username),
        })?;

        get_other_collection(state, get).await
}

pub async fn get_all_sector(state: Arc<AppState>) -> DbClassResult<Vec<SectorModelGet>> {
    let get = state
        .db
        .sector
        .get_many(None, Some("sector".to_string()))
        .await?;

    let mut sectors: Vec<SectorModelGet> = Vec::new();

    for sector in get {
        sectors.push( get_other_collection(state.clone(), sector).await?);
    }
    Ok(sectors)
}

pub async fn get_all_sector_by_education(state: Arc<AppState>, id : ObjectId) -> DbClassResult<Vec<SectorModelGet>> {
    let get = state
        .db
        .sector
        .get_many(Some(GetManyByField{ field: "education_id".to_string() , value : id}), Some("sector".to_string()))
        .await?;

    let mut sectors: Vec<SectorModelGet> = Vec::new();

    for sector in get {
        sectors.push( get_other_collection(state.clone(), sector).await?);
    }
    Ok(sectors)
}

pub async fn get_sector_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<SectorModelGet> {
    let get = state
        .db
        .sector
        .get_one_by_id(id, Some("sector".to_string()))
        .await?;
    get_other_collection(state, get).await
}

pub async fn update_sector_by_id(
    state: Arc<AppState>,
    id: ObjectId,
   mut sector: SectorModelPut,
) -> DbClassResult<SectorModelGet> {
    if let Some(ref username) = sector.username {
        let _ = validate_sector_username(state.clone(), username, Some(id)).await;
        }

    if let Some(ref education) = sector.education {
        let id = match ObjectId::from_str(education) {
            Err(_) => {
                return Err(DbClassError::OtherError {
                    err: format!("Invalid education id [{}], please try other id", education),
                })
            }
            Ok(i) => i,
        };

        let get_education = get_education_by_id(state.clone(), id).await;

        if get_education.is_err() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Education id is not found [{}], please try other id",
                    education
                ),
            });
        }
    }

    let exit_sector = get_sector_by_id(state.clone(), id).await?;

    if let Some(file) = sector.symbol {
        sector.symbol = Some(handle_symbol_update(state.clone(), file, exit_sector.symbol).await?);
    }
    
    state
        .db
        .sector
        .update(id, SectorModel::put(sector), Some("sector".to_string()))
        .await?;

    let get = get_sector_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_sector_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SectorModelGet> {
    let get_trade = get_trade_by_sector(state.clone(),id).await;

if let Ok(sectors) = get_trade {
    if !sectors.is_empty() {
        return Err(DbClassError::OtherError { 
            err: "You cannot delete sector account because there are trades associated with it. If you need to delete it, delete those trade accounts first.".to_string() 
        });
    }
}
let get = get_sector_by_id(state.clone(), id).await?;
    let _ = state
        .db
        .sector
        .delete(id, Some("sector".to_string()))
        .await?;

    Ok(get)
}
