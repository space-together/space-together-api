use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
    AppState,
};

pub async fn create_class(
    state: Arc<AppState>,
    class: ClassModelNew,
) -> DbClassResult<ClassModelGet> {
    let create = state
        .db
        .class
        .create(
            ClassModel::new(class),
            Some("class".to_string()),
        )
        .await?;
    let get = state
        .db
        .class
        .get_one_by_id(create, Some("class".to_string()))
        .await?;
    Ok(ClassModel::format(get))
}


pub async fn get_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .get_one_by_id(id, Some("class".to_string()))
        .await?;
    Ok(ClassModel::format(get))
}

pub async fn get_all_class(state: Arc<AppState>) -> DbClassResult<Vec<ClassModelGet>> {
    let get = state
        .db
        .class
        .get_many(None, Some("class".to_string()))
        .await?;
    Ok(get.into_iter().map(ClassModel::format).collect())
}

pub async fn update_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    class: ClassModelPut,
) -> DbClassResult<ClassModelGet> {
    let _ = state
        .db
        .class
        .update(
            id,
            ClassModel::put(class),
            Some("class".to_string()),
        )
        .await;
    let get = get_class_by_id(state, id).await?;
    Ok(get)
}

pub async fn delete_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassModelGet> {
    let delete = state
        .db
        .class
        .delete(id, Some("class".to_string()))
        .await?;
    Ok(ClassModel::format(delete))
}
