use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::activities_error::ActivitiesResult,
    models::class_model::activity_model::{ActivityModel, ActivityModelGet},
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
