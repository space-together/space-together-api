use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::DbClassResult,
    models::class_model::class_type_model::{
        ClassTypeModel, ClassTypeModelGet, ClassTypeModelNew, ClassTypeModelPut,
    },
    AppState,
};

pub async fn create_class_type(
    state: Arc<AppState>,
    class_type: ClassTypeModelNew,
) -> DbClassResult<ClassTypeModelGet> {
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
