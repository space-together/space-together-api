use std::{str::FromStr, sync::Arc};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::class_controller::{
        class_room_controller::get_class_room_by_id,
        class_room_type_controller::get_class_room_type_by_id,
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::classes::db_crud::GetManyByField,
    models::subject_model::subject_model_model::{
        SubjectModel, SubjectModelGet, SubjectModelNew, SubjectModelPut,
    },
    AppState,
};

async fn get_other_collection(
    state: Arc<AppState>,
    item: SubjectModel,
) -> DbClassResult<SubjectModelGet> {
    let mut subject = SubjectModel::format(item.clone());
    if let Some(id) = item.class_room_id {
        let class_room = get_class_room_by_id(state, id).await?;
        subject.class_room_id = class_room.username.or(Some(class_room.name))
    }
    Ok(subject)
}

pub async fn create_subject(
    state: Arc<AppState>,
    subject: SubjectModelNew,
) -> DbClassResult<SubjectModelGet> {
    if let Some(exit_code) = subject.code.clone() {
        if get_subject_by_code(state.clone(), &exit_code)
            .await
            .is_err()
        {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Subject code is ready exit [{}], please try other",
                    exit_code
                ),
            });
        }
    }
    let index = IndexModel::builder()
        .keys(doc! {
        "code" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if state
        .db
        .subject
        .collection
        .create_index(index)
        .await
        .is_err()
    {
        return Err(DbClassError::OtherError {
            err: "Subject Code is ready exit, try other code".to_string(),
        });
    }

    if let Some(class_room) = subject.class_room_id.clone() {
        if let Ok(id) = ObjectId::from_str(&class_room) {
            if get_class_room_by_id(state.clone(), id).await.is_ok() {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "your class room id is not found  [{}], please use other class room id",
                        class_room
                    ),
                });
            }
        } else {
            return Err(DbClassError::OtherError {
                err: format!("Your class room id is invalid [{}], try other", class_room),
            });
        }
    }

    let create = state
        .db
        .subject
        .create(SubjectModel::new(subject), Some("subject".to_string()))
        .await?;
    let get = get_subject_by_id(state, create).await?;
    Ok(get)
}

pub async fn get_all_subject(state: Arc<AppState>) -> DbClassResult<Vec<SubjectModelGet>> {
    let get = state
        .db
        .subject
        .get_many(None, Some("subject".to_string()))
        .await?;
    let mut subjects = Vec::new();
    for subject in get {
        let format = get_other_collection(state.clone(), subject).await?;
        subjects.push(format);
    }

    Ok(subjects)
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
    let format = get_other_collection(state, get).await?;
    Ok(format)
}

pub async fn get_subject_by_code(
    state: Arc<AppState>,
    code: &String,
) -> DbClassResult<SubjectModelGet> {
    let get = state
        .db
        .subject
        .collection
        .find_one(doc! {"code" :code })
        .await;
    match get {
        Ok(Some(item)) => Ok(get_other_collection(state, item).await?),
        Ok(None) => Err(DbClassError::OtherError {
            err: format!(
                "Can't get subject by code [{}], please try other code",
                code
            ),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Some thing went wrong to get subject but code , try again later".to_string(),
        }),
    }
}

pub async fn get_subjects_by_class_room(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<Vec<SubjectModelGet>> {
    let get = state
        .db
        .subject
        .get_many(
            Some(GetManyByField {
                field: "class_room_id".to_string(),
                value: id,
            }),
            Some("Subjects".to_string()),
        )
        .await?;
    let mut subjects = Vec::new();
    for subject in get {
        let format = get_other_collection(state.clone(), subject).await?;
        subjects.push(format);
    }

    Ok(subjects)
}

pub async fn update_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    subject: SubjectModelPut,
) -> DbClassResult<SubjectModelGet> {
    if let Some(class_room) = subject.class_room_id.clone() {
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
