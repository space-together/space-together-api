use std::sync::Arc;

use crate::{
    error::class_error::activities_type_error::{ActivitiesTypeErr, ActivitiesTypeResult},
    models::class_model::activities_type_model::{
        ActivitiesTypeModel, ActivitiesTypeModelGet, ActivitiesTypeModelNew,
    },
    AppState,
};

pub async fn controller_activities_type_create(
    state: Arc<AppState>,
    ty: ActivitiesTypeModelNew,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    let create = state.db.activities_type.create_activity_type(ty).await;
    match create {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ActivitiesTypeErr::InvalidId)
                .unwrap();
            let get = state.db.activities_type.get_activities_type_by_id(id).await;
            match get {
                Ok(res) => Ok(ActivitiesTypeModel::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_activities_type_get_by_id(
    state: Arc<AppState>,
    id: String,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    let get = state.db.activities_type.get_activities_type_by_id(id).await;
    match get {
        Ok(res) => Ok(ActivitiesTypeModel::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_activities_type_get_all(
    state: Arc<AppState>,
) -> ActivitiesTypeResult<Vec<ActivitiesTypeModelGet>> {
    let get = state.db.activities_type.get_all_activities_type().await;
    match get {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}
