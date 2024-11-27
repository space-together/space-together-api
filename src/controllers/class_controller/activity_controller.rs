use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::activities_error::{ActivitiesErr, ActivitiesResult},
    libs::functions::{
        characters_fn::validate_datetime, object_id::change_insertoneresult_into_object_id,
    },
    models::class_model::activity_model::{
        ActivityModel, ActivityModelGet, ActivityModelNew, ActivityModelPut,
    },
    AppState,
};

pub async fn controller_activity_get_by_class(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesResult<Vec<ActivityModelGet>> {
    match state.db.activity.get_activity_by_class(id).await {
        Err(err) => Err(err),
        Ok(res) => Ok(res.into_iter().map(ActivityModel::format).collect()),
    }
}

pub async fn controller_activity_get_by_group(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesResult<Vec<ActivityModelGet>> {
    match state.db.activity.get_activity_by_group(id).await {
        Err(err) => Err(err),
        Ok(res) => Ok(res.into_iter().map(ActivityModel::format).collect()),
    }
}

pub async fn controller_activity_get_by_teacher(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesResult<Vec<ActivityModelGet>> {
    match state.db.activity.get_activity_by_teacher(id).await {
        Err(err) => Err(err),
        Ok(res) => Ok(res.into_iter().map(ActivityModel::format).collect()),
    }
}

pub async fn controller_activity_create(
    state: Arc<AppState>,
    activity: ActivityModelNew,
) -> ActivitiesResult<ActivityModelGet> {
    if let Err(err) = validate_datetime(&activity.dl) {
        return Err(ActivitiesErr::InvalidDateTime { date: err });
    }

    if ObjectId::from_str(&activity.ty).is_err() {
        return Err(ActivitiesErr::Invalid);
    }

    if state
        .db
        .activities_type
        .get_activities_type_by_id(ObjectId::from_str(&activity.ty).unwrap())
        .await
        .is_err()
    {
        return Err(ActivitiesErr::ActivityTypeIsNotExit);
    }

    if activity.cl.is_some() && activity.gr.is_some() {
        return Err(ActivitiesErr::ClassAndActivityCanNotHaveOneActivity);
    }

    match state.db.activity.create_activity(activity).await {
        Err(err) => Err(err),
        Ok(res) => match state
            .db
            .activity
            .get_activity_by_id(change_insertoneresult_into_object_id(res))
            .await
        {
            Err(err) => Err(err),
            Ok(activity) => Ok(ActivityModel::format(activity)),
        },
    }
}

pub async fn controller_activity_get_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesResult<ActivityModelGet> {
    match state.db.activity.get_activity_by_id(id).await {
        Err(err) => Err(err),
        Ok(activity) => Ok(ActivityModel::format(activity)),
    }
}

pub async fn controller_activity_delete_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ActivitiesResult<ActivityModelGet> {
    match state.db.activity.delete_activity_by_id(id).await {
        Err(err) => Err(err),
        Ok(activity) => Ok(ActivityModel::format(activity)),
    }
}

pub async fn controller_activity_update_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    activity: ActivityModelPut,
) -> ActivitiesResult<ActivityModelGet> {
    if let Some(date) = activity.dl.clone() {
        if let Err(err) = validate_datetime(&date) {
            return Err(ActivitiesErr::InvalidDateTime { date: err });
        }
    }

    match state.db.activity.update_activity_by_id(id, activity).await {
        Err(err) => Err(err),
        Ok(_) => match state.db.activity.get_activity_by_id(id).await {
            Err(err) => Err(err),
            Ok(activity) => Ok(ActivityModel::format(activity)),
        },
    }
}
