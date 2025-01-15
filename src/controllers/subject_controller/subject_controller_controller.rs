use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    controllers::class_controller::{
        class_room_controller::get_class_room_by_id,
        class_room_type_controller::get_class_room_type_by_id,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    models::subject_model::subject_model_model::{
        SubjectModel, SubjectModelGet, SubjectModelNew, SubjectModelPut,
    },
    AppState,
};

pub async fn create_subject(
    state: Arc<AppState>,
    subject: SubjectModelNew,
) -> DbClassResult<SubjectModelGet> {
    if let Some(class_room) = subject.room_id.clone() {
        let id = ObjectId::from_str(&class_room);
        if id.is_err() {
            return Err(DbClassError::OtherError {
                err: format!("Your class room id is invalid [{}], try other", class_room),
            });
        };

        if let Ok(i) = id {
            let get = get_class_room_type_by_id(state.clone(), i).await;
            if get.is_ok() {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "your class room id is not found  [{}], please use other class room id",
                        class_room
                    ),
                });
            }
        }
    }

    if let Some(subject_type) = subject.subject_type_id.clone() {
        let id = ObjectId::from_str(&subject_type);
        if id.is_err() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Your subject type id is invalid [{}], try other",
                    subject_type
                ),
            });
        };

        if let Ok(i) = id {
            let get = get_class_room_by_id(state.clone(), i).await;
            if let Err(e) = get {
                return Err(DbClassError::OtherError { err: e.to_string() });
            }
        }
    }

    let create = state
        .db
        .subject
        .create(SubjectModel::new(subject), Some("subject".to_string()))
        .await?;
    let get = state
        .db
        .subject
        .get_one_by_id(create, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(get))
}

pub async fn get_all_subject(state: Arc<AppState>) -> DbClassResult<Vec<SubjectModelGet>> {
    let get = state
        .db
        .subject
        .get_many(None, Some("subject".to_string()))
        .await?;
    Ok(get.into_iter().map(SubjectModel::format).collect())
}

pub async fn get_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectModelGet> {
    let get = state
        .db
        .subject
        .get_one_by_id(id, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(get))
}

pub async fn update_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    subject: SubjectModelPut,
) -> DbClassResult<SubjectModelGet> {
    if let Some(class_room) = subject.room_id.clone() {
        let id = ObjectId::from_str(&class_room);
        if id.is_err() {
            return Err(DbClassError::OtherError {
                err: format!("Your class room id is invalid [{}], try other", class_room),
            });
        };

        if let Ok(i) = id {
            let get = get_class_room_type_by_id(state.clone(), i).await;
            if get.is_ok() {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "your class room id is not found  [{}], please use other class room id",
                        class_room
                    ),
                });
            }
        }
    }

    if let Some(subject_type) = subject.subject_type_id.clone() {
        let id = ObjectId::from_str(&subject_type);
        if id.is_err() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Your subject type id is invalid [{}], try other",
                    subject_type
                ),
            });
        };

        if let Ok(i) = id {
            let get = get_class_room_by_id(state.clone(), i).await;
            if let Err(e) = get {
                return Err(DbClassError::OtherError { err: e.to_string() });
            }
        }
    }

    let _ = state
        .db
        .subject
        .update(id, SubjectModel::put(subject), Some("subject".to_string()))
        .await;
    let get = get_subject_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectModelGet> {
    let delete = state
        .db
        .subject
        .delete(id, Some("subject".to_string()))
        .await?;
    Ok(SubjectModel::format(delete))
}
