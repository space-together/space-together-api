use std::{str::FromStr, sync::Arc};

use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    controllers::user_controller::user_controller_controller::{
        controller_get_user_by_id, get_user_by_token,
    },
    models::{
        school_model::school_model_model::{
            SchoolModel, SchoolModelGet, SchoolModelNew, SchoolModelPut,
        },
        user_model::user_model_model::UserRole,
    },
    AppState,
};

pub async fn create_school(
    state: &Arc<AppState>,
    school: &SchoolModelNew,
    token: &str,
) -> Result<SchoolModelGet, String> {
    let user = get_user_by_token(token, state).await?;
    if user.role != Some(UserRole::SCHOOLSTAFF) || user.role != Some(UserRole::ADMIN) {
        return Err("You are not allowed to create a school".to_string());
    }
    let creator_id = ObjectId::from_str(&school.creator_id)
        .map_err(|_| "Creator id is not a valid ObjectId".to_string())?;
    let get_creator = controller_get_user_by_id(state.clone(), creator_id)
        .await
        .map_err(|e| e.to_string())?;
    if get_creator.role != Some(UserRole::SCHOOLSTAFF) && get_creator.role != Some(UserRole::ADMIN)
    {
        return Err("You are not allowed to create a school".to_string());
    }

    let school_id = state
        .db
        .school
        .create(SchoolModel::new(school.clone()), Some("school".to_string()))
        .await
        .map_err(|e| e.to_string())?;
    state
        .db
        .school
        .get_one_by_id(school_id, Some("School".to_string()))
        .await
        .map_err(|e| e.to_string())
        .map(|d| SchoolModel::format(&d))
}

pub async fn get_school_by_id(
    state: &Arc<AppState>,
    id: ObjectId,
) -> Result<SchoolModelGet, String> {
    state
        .db
        .school
        .get_one_by_id(id, Some("school".to_string()))
        .await
        .map_err(|e| e.to_string())
        .map(|d| SchoolModel::format(&d))
}

pub async fn update_school_by_id(
    state: &Arc<AppState>,
    id: ObjectId,
    school: &SchoolModelPut,
) -> Result<SchoolModelGet, String> {
    let update = state
        .db
        .school
        .collection
        .find_one_and_update(
            doc! {"_id": id},
            doc! {"$set" : SchoolModel::put(school.clone())},
        )
        .await
        .map_err(|e| e.to_string())?;

    match update {
        Some(d) => Ok(get_school_by_id(state, d.id.unwrap()).await?),
        None => Err("School not found".to_string()),
    }
}
