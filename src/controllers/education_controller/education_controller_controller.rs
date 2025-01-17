use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{classes::db_crud::GetManyByField, functions::characters_fn::is_valid_username},
    models::education_model::education_model_model::{
        EducationModel, EducationModelGet, EducationModelNew, EducationModelPut,
    },
    AppState,
};

pub async fn create_education(
    state: Arc<AppState>,
    education: EducationModelNew,
) -> DbClassResult<EducationModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {
        "username" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(err) = state.db.education.collection.create_index(index).await {
        return Err(DbClassError::OtherError {
            err: format!(
                "Can't create education bcs username is leady exit 😡 [{}]😡 ",
                err
            ),
        });
    }

    if let Some(ref username) = education.username {
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_education_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username education is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    } else {
        return Err(DbClassError::OtherError {
            err: "Username is missing".to_string(),
        });
    }

    let create = state
        .db
        .education
        .create(
            EducationModel::new(education),
            Some("education".to_string()),
        )
        .await?;
    let get = get_education_by_id(state, create).await?;

    Ok(get)
}

pub async fn get_all_education(state: Arc<AppState>) -> DbClassResult<Vec<EducationModelGet>> {
    let get = state
        .db
        .education
        .get_many(None, Some("education".to_string()))
        .await?;
    Ok(get.into_iter().map(EducationModel::format).collect())
}

pub async fn get_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<EducationModelGet> {
    let get = state
        .db
        .education
        .get_one_by_id(id, Some("education".to_string()))
        .await?;
    Ok(EducationModel::format(get))
}

pub async fn get_education_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<EducationModelGet> {
    let get = state
        .db
        .education
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Education not found by username [{}]", &username),
        })?;

    Ok(EducationModel::format(get))
}

pub async fn update_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    education: EducationModelPut,
) -> DbClassResult<EducationModelGet> {
    if let Some(username) = &education.username {
        let get_username = get_education_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username education is ready exit [{}], please try other",
                    &username
                ),
            });
        }
    };

    let _ = state
        .db
        .education
        .update(
            id,
            EducationModel::put(education),
            Some("education".to_string()),
        )
        .await;
    let get = get_education_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_education_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<EducationModelGet> {
    let get_sectors = state
        .db
        .sector
        .get_many(
            Some(GetManyByField {
                field: "education_id".to_string(),
                value: id,
            }),
            Some("education".to_string()),
        )
        .await;

    if let Ok(sectors) = get_sectors {
        if !sectors.is_empty() {
            return Err(DbClassError::OtherError { 
                err: "You cannot delete education account because there are trades associated with it. If you need to delete it, delete those sectors accounts first.".to_string() 
            });
        }
    }

    let get = get_education_by_id(state.clone(), id).await?;
    let _ = state
        .db
        .education
        .delete(id, Some("education".to_string()))
        .await?;
    Ok(get)
}
