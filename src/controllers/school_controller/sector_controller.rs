use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::education_controller::education_controller_controller::get_education_by_id,
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{classes::db_crud::GetManyByField, functions::characters_fn::is_valid_username},
    models::school_model::sector_model::{
        SectorModel, SectorModelGet, SectorModelNew, SectorModelPut,
    },
    AppState,
};

pub async fn create_sector(
    state: Arc<AppState>,
    sector: SectorModelNew,
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
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_sector_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username education is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    } else {
        return Err(DbClassError::OtherError {
            err: "Username is missing".to_string(),
        });
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

        let mut education_name: Option<String> = None;

        if let Some(ref education_id) = get.education_id {
            let get_education = get_education_by_id(state.clone(), *education_id).await?;
    
            if let Some(education_username) = get_education.username {
                education_name = Some(education_username);
            } else {
                education_name = Some(get_education.name);
            }
        }
        let mut sector = SectorModel::format(get);
        sector.education = education_name;
        Ok(sector)
}

pub async fn get_all_sector(state: Arc<AppState>) -> DbClassResult<Vec<SectorModelGet>> {
    let get = state
        .db
        .sector
        .get_many(None, Some("sector".to_string()))
        .await?;

    let mut sectors: Vec<SectorModelGet> = Vec::new();

    for sector in get {
        if let Some(ref education_id) = sector.education_id {
            let get_education = get_education_by_id(state.clone(), *education_id).await?;
            if let Some(education_username) = get_education.username {
                let mut fol_sector = SectorModel::format(sector);
                fol_sector.education = Some(education_username);
                sectors.push(fol_sector);
            } else {
                let mut fol_sector = SectorModel::format(sector);
                fol_sector.education = Some(get_education.name);
                sectors.push(fol_sector);
            }
        }
    }
    Ok(sectors)
}

pub async fn get_sector_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<SectorModelGet> {
    let get = state
        .db
        .sector
        .get_one_by_id(id, Some("sector".to_string()))
        .await?;
    let mut education_name: Option<String> = None;

    if let Some(ref education_id) = get.education_id {
        let get_education = get_education_by_id(state.clone(), *education_id).await?;

        if let Some(education_username) = get_education.username {
            education_name = Some(education_username);
        } else {
            education_name = Some(get_education.name);
        }
    }
    let mut sector = SectorModel::format(get);
    sector.education = education_name;
    Ok(sector)
}

pub async fn update_sector_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    sector: SectorModelPut,
) -> DbClassResult<SectorModelGet> {
    if let Some(ref username) = sector.username {
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_sector_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username education is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    }
    let _ = state
        .db
        .sector
        .update(id, SectorModel::put(sector), Some("sector".to_string()))
        .await;
    let get = get_sector_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_sector_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SectorModelGet> {
    let get_trade = state
    .db
    .sector
    .get_many(
        Some(GetManyByField {
            field: "trade_id".to_string(),
            value: id,
        }),
        Some("trade".to_string()),
    )
    .await;

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
