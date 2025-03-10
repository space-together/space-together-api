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
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

// Function to get single subject with class room details
async fn get_other_collection(
    state: Arc<AppState>,
    item: SubjectModel,
) -> DbClassResult<SubjectModelGet> {
    let mut subject = SubjectModel::format(item.clone());
    if let Some(id) = item.class_room_id {
        let class_room = get_class_room_by_id(state, id).await?;
        subject.class_room_id = class_room.username.or(Some(class_room.name));
    }
    Ok(subject)
}

// Optimized batch fetch function
async fn get_many_other_collection(
    state: Arc<AppState>,
    items: Vec<SubjectModel>,
) -> DbClassResult<Vec<SubjectModelGet>> {
    let mut subjects: Vec<SubjectModelGet> = Vec::new();

    // Collect unique class_room_ids
    let class_room_ids: Vec<ObjectId> =
        items.iter().filter_map(|item| item.class_room_id).collect();

    // Fetch all class rooms in one query
    let mut cursor = state
        .db
        .class_room
        .collection
        .find(doc! {"_id": { "$in": &class_room_ids } })
        .await?;

    let mut class_rooms_map = HashMap::new();

    while let Some(room) = cursor.next().await {
        if let Ok(room) = room {
            if let Some(id) = room.id {
                // Ensure `id` is not `Option<ObjectId>`
                class_rooms_map.insert(id, room.username.or(Some(room.name)));
            }
        }
    }

    for item in items {
        let mut subject = SubjectModel::format(item.clone());
        if let Some(id) = item.class_room_id {
            // No `.as_ref()`, just use `id`
            subject.class_room_id = class_rooms_map.get(&id).cloned().flatten();
        }
        subjects.push(subject);
    }

    Ok(subjects)
}

// Create a new subject
pub async fn create_subject(
    state: Arc<AppState>,
    subject: SubjectModelNew,
) -> DbClassResult<SubjectModelGet> {
    // Check if subject code exists
    if let Some(exit_code) = subject.code.clone() {
        if let Ok(code) = get_subject_by_code(state.clone(), &exit_code).await {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Subject code [{:?}] already exists, please try another",
                    code.code
                ),
            });
        }
    }

    // Validate class_room_id
    if let Some(class_room) = subject.class_room_id.clone() {
        let class_room_id =
            ObjectId::from_str(&class_room).map_err(|_| DbClassError::OtherError {
                err: format!("Invalid class room ID [{}], try another", class_room),
            })?;
        get_class_room_by_id(state.clone(), class_room_id).await?;
    }

    // Insert subject
    let created_subject = state
        .db
        .subject
        .create(SubjectModel::new(subject), Some("subject".to_string()))
        .await?;
    get_subject_by_id(state, created_subject).await
}

// Fetch all subjects
pub async fn get_all_subject(state: Arc<AppState>) -> DbClassResult<Vec<SubjectModelGet>> {
    let get = state
        .db
        .subject
        .get_many(None, Some("subject".to_string()))
        .await?;
    get_many_other_collection(state.clone(), get).await
}

// Fetch subject by ID
pub async fn get_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SubjectModelGet> {
    let get = state
        .db
        .subject
        .get_one_by_id(id, Some("subject".to_string()))
        .await?;
    get_other_collection(state, get).await
}

// Fetch subject by code
pub async fn get_subject_by_code(
    state: Arc<AppState>,
    code: &String,
) -> DbClassResult<SubjectModelGet> {
    let get = state
        .db
        .subject
        .collection
        .find_one(doc! {"code": code })
        .await;

    match get {
        Ok(Some(item)) => get_other_collection(state, item).await,
        Ok(None) => Err(DbClassError::OtherError {
            err: format!("No subject found with code [{}]", code),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Error retrieving subject by code, try again later".to_string(),
        }),
    }
}

// Fetch subjects by class room
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
    get_many_other_collection(state.clone(), get).await
}

// Update subject by ID
pub async fn update_subject_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    subject: SubjectModelPut,
) -> DbClassResult<SubjectModelGet> {
    // Validate class_room_id if provided
    if let Some(class_room) = subject.class_room_id.clone() {
        let id = ObjectId::from_str(&class_room).map_err(|_| DbClassError::OtherError {
            err: format!("Invalid class room ID [{}], try another", class_room),
        })?;

        if get_class_room_type_by_id(state.clone(), id).await.is_err() {
            return Err(DbClassError::OtherError {
                err: format!("Class room ID [{}] not found, use another", class_room),
            });
        }
    }

    // Update subject
    let _ = state
        .db
        .subject
        .update(id, SubjectModel::put(subject), Some("subject".to_string()))
        .await;
    get_subject_by_id(state, id).await
}

// Delete subject by ID
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
