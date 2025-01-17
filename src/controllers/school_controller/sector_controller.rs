use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::characters_fn::is_valid_username,
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
                "Can't create education bcs username is leady exit 😡 [{}]😡 ",
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

    Ok(SectorModel::format(get))
}

pub async fn get_all_sector(state: Arc<AppState>) -> DbClassResult<Vec<SectorModelGet>> {
    let get = state
        .db
        .sector
        .get_many(None, Some("sector".to_string()))
        .await?;
    Ok(get.into_iter().map(SectorModel::format).collect())
}

pub async fn get_sector_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<SectorModelGet> {
    let get = state
        .db
        .sector
        .get_one_by_id(id, Some("sector".to_string()))
        .await?;
    Ok(SectorModel::format(get))
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
    let delete = state
        .db
        .sector
        .delete(id, Some("sector".to_string()))
        .await?;
    Ok(SectorModel::format(delete))
}
