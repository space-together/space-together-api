use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    models::school_model::school_model_model::{SchoolModel, SchoolModelGet, SchoolModelNew},
    AppState,
};

pub async fn controller_school_create(
    state: Arc<AppState>,
    school: SchoolModelNew,
) -> DbClassResult<SchoolModelGet> {
    let owner_id = match ObjectId::from_str(&school.owner) {
        Err(_) => return Err(DbClassError::InvalidId),
        Ok(e) => e,
    };
    let get_owner = state.db.user.get_user_by_id(owner_id).await;

    if let Err(e) = get_owner {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    let collection = Some("School".to_string());
    let create = state
        .db
        .school
        .create(SchoolModel::new(school), collection.clone())
        .await;
    match create {
        Err(e) => Err(e),
        Ok(i) => match state.db.school.get_one_by_id(i, collection.clone()).await {
            Err(err) => Err(err),
            Ok(k) => Ok(SchoolModel::format(k)),
        },
    }
}

pub async fn controller_school_get(state: Arc<AppState>) -> DbClassResult<Vec<SchoolModelGet>> {
    let collection = Some("School".to_string());
    let get = state.db.school.get_many(None, collection).await;
    match get {
        Err(e) => Err(e),
        Ok(k) => Ok(k.into_iter().map(SchoolModel::format).collect()),
    }
}

pub async fn controller_school_get_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SchoolModelGet> {
    let collection = Some("School".to_string());
    let get = state.db.school.get_one_by_id(id, collection).await;
    match get {
        Err(e) => Err(e),
        Ok(k) => Ok(SchoolModel::format(k)),
    }
}

// pub async fn controller_school_update_by_id(state: Arc<AppState> , id: ObjectId , school: SchoolModelPut)
