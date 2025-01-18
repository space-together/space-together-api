use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::characters_fn::is_valid_username,
    models::class_model::class_type_model::{
        ClassTypeModel, ClassTypeModelGet, ClassTypeModelNew, ClassTypeModelPut,
    },
    AppState,
};

pub async fn create_class_type(
    state: Arc<AppState>,
    class_type: ClassTypeModelNew,
) -> DbClassResult<ClassTypeModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {
        "username" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(err) = state.db.education.collection.create_index(index).await {
        return Err(DbClassError::OtherError {
            err: format!(
                "Can't create education bcs username is leady exit  [{}] ",
                err
            ),
        });
    }

    if let Some(ref username) = class_type.username {
        if let Err(err) = is_valid_username(username) {
            return Err(DbClassError::OtherError {
                err: err.to_string(),
            });
        }

        let get_username = get_class_type_by_username(state.clone(), username.clone()).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username class type is ready exit [{}], please try other",
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
        .class_type
        .create(
            ClassTypeModel::new(class_type),
            Some("class_type".to_string()),
        )
        .await?;
    let get = state
        .db
        .class_type
        .get_one_by_id(create, Some("class_type".to_string()))
        .await?;
    Ok(ClassTypeModel::format(get))
}

pub async fn get_class_type_by_username(
    state: Arc<AppState>,
    username: String,
) -> DbClassResult<ClassTypeModelGet> {
    let get = state
        .db
        .class_type
        .collection
        .find_one(doc! {"username" : &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Sector not found by username [{}]", &username),
        })?;

    Ok(ClassTypeModel::format(get))
}

pub async fn get_all_class_type(state: Arc<AppState>) -> DbClassResult<Vec<ClassTypeModelGet>> {
    let get = state
        .db
        .class_type
        .get_many(None, Some("class_type".to_string()))
        .await?;
    Ok(get.into_iter().map(ClassTypeModel::format).collect())
}

pub async fn get_class_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassTypeModelGet> {
    let get = state
        .db
        .class_type
        .get_one_by_id(id, Some("class_type".to_string()))
        .await?;
    Ok(ClassTypeModel::format(get))
}

pub async fn update_class_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class_type: ClassTypeModelPut,
) -> DbClassResult<ClassTypeModelGet> {
    let _ = state
        .db
        .class_type
        .update(
            id,
            ClassTypeModel::put(class_type),
            Some("class_type".to_string()),
        )
        .await;
    let get = get_class_type_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_class_type_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassTypeModelGet> {
    let delete = state
        .db
        .class_type
        .delete(id, Some("class_type".to_string()))
        .await?;
    Ok(ClassTypeModel::format(delete))
}
