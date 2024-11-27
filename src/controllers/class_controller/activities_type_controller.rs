use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::activities_type_error::{ActivitiesTypeErr, ActivitiesTypeResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::class_model::activities_type_model::{
        ActivitiesTypeModel, ActivitiesTypeModelGet, ActivitiesTypeModelNew, ActivitiesTypeModelPut,
    },
    AppState,
};

pub async fn controller_activities_type_create(
    state: Arc<AppState>,
    ty: ActivitiesTypeModelNew,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    if let Ok(name) = state
        .db
        .activities_type
        .get_activities_type_by_ty(ty.ty.clone())
        .await
    {
        return Err(ActivitiesTypeErr::ActivitiesTypeIsReadyExit { name: name.ty });
    };

    match state.db.activities_type.create_activity_type(ty).await {
        Ok(i) => {
            let get = state
                .db
                .activities_type
                .get_activities_type_by_id(change_insertoneresult_into_object_id(i))
                .await;
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
    id: ObjectId,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    let get = state.db.activities_type.get_activities_type_by_id(id).await;
    match get {
        Ok(res) => Ok(ActivitiesTypeModel::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_activities_type_delete_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    match state
        .db
        .activities_type
        .delete_activities_type_by_id(id)
        .await
    {
        Ok(res) => Ok(ActivitiesTypeModel::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_activities_type_update_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    activity_type: ActivitiesTypeModelPut,
) -> ActivitiesTypeResult<ActivitiesTypeModelGet> {
    if let Some(ty) = activity_type.ty.clone() {
        if let Ok(ty) = state.db.activities_type.get_activities_type_by_ty(ty).await {
            return Err(ActivitiesTypeErr::ActivitiesTypeIsReadyExit { name: ty.ty });
        }
    }

    match state
        .db
        .activities_type
        .update_activities_type_by_id(id, activity_type)
        .await
    {
        Ok(res) => match state
            .db
            .activities_type
            .get_activities_type_by_id(res.id.unwrap())
            .await
        {
            Ok(result) => Ok(ActivitiesTypeModel::format(result)),
            Err(err) => Err(err),
        },
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
