use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::class_error_error::{ClassError, ClassResult},
    models::class_model::class_model_model::{ClassModelGet, ClassModelNew},
    AppState,
};

pub async fn controller_create_class(
    state: Arc<AppState>,
    class: ClassModelNew,
) -> ClassResult<ClassModelGet> {
    let find_user = state
        .db
        .user
        .get_user_by_id(ObjectId::from_str(&class.cltea).unwrap())
        .await;
    if find_user.is_err() {
        return Err(ClassError::ClassTeacherIsNotExit);
    }
    match state.db.class.create_class(class).await {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ClassError::InvalidId)
                .unwrap();
            let get = state.db.class.get_class_by_id(id).await;
            match get {
                Ok(res) => Ok(ClassModelGet::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_get_class_by_id(
    state: Arc<AppState>,
    id: String,
) -> ClassResult<ClassModelGet> {
    let get = state.db.class.get_class_by_id(id).await;
    match get {
        Ok(res) => Ok(ClassModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_classes(state: Arc<AppState>) -> ClassResult<Vec<ClassModelGet>> {
    let all_classes = state.db.class.get_all_classes().await;
    match all_classes {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}
