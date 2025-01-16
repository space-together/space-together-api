use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::school_model::sector_model::{
        SectorModel, SectorModelGet, SectorModelNew, SectorModelPut,
    },
    AppState,
};

pub async fn create_sector(
    state: Arc<AppState>,
    sector: SectorModelNew,
) -> DbClassResult<SectorModelGet> {
    let create = state
        .db
        .sector
        .create(SectorModel::new(sector), Some("sector".to_string()))
        .await?;
    let get = state
        .db
        .sector
        .get_one_by_id(create, Some("sector".to_string()))
        .await?;
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
