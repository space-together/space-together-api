use std::sync::Arc;

use crate::{
    error::class_error::class_group_err::{ClassGroupErr, ClassGroupResult},
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
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ClassGroupErr::InvalidId)
                .unwrap();
            let get = state.db.class_group.get_class_group_by_id(id).await;
            match get {
                Ok(res) => Ok(ClassGroupModel::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

// pub
