use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::class_group_err::ClassGroupResult,
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::class_model::class_group_model::class_group_model_model::{
        ClassGroupModel, ClassGroupModelGet, ClassGroupModelNew,
    },
    AppState,
};

pub async fn controller_class_group_create(
    state: Arc<AppState>,
    group: ClassGroupModelNew,
) -> ClassGroupResult<ClassGroupModelGet> {
    let create = state.db.class_group.class_group_create(group).await;
    match create {
        Ok(_id) => {
            let get = state
                .db
                .class_group
                .get_class_group_by_id(change_insertoneresult_into_object_id(_id))
                .await;
            match get {
                Ok(res) => Ok(ClassGroupModel::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_class_group_get_all(
    state: Arc<AppState>,
) -> ClassGroupResult<Vec<ClassGroupModelGet>> {
    let all = state.db.class_group.get_all_class_group().await;
    match all {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_class_group_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ClassGroupResult<ClassGroupModelGet> {
    match state.db.class_group.get_class_group_by_id(id).await {
        Ok(res) => Ok(ClassGroupModel::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_class_groups_by_class(
    state: Arc<AppState>,
    id: ObjectId,
) -> ClassGroupResult<Vec<ClassGroupModelGet>> {
    match state.db.class_group.get_class_group_by_class(id).await {
        Ok(res) => Ok(res.into_iter().map(ClassGroupModel::format).collect()),
        Err(err) => Err(err),
    }
}
