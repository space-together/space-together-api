use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::activities_error::{ActivitiesErr, ActivitiesResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::class_model::activity_model::{ActivityModel, ActivityModelGet, ActivityModelNew},
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

pub async fn controller_activity_create(
    state: Arc<AppState>,
    activity: ActivityModelNew,
) -> ActivitiesResult<ActivityModelGet> {
    if state
        .db
        .activities_type
        .get_activities_type_by_id(activity.ty.clone())
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
