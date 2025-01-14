use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    models::school_model::school_section_model::{
        SchoolSectionModel, SchoolSectionModelGet, SchoolSectionModelNew, SchoolSectionModelPut,
    },
    AppState,
};

pub async fn create_school_section(
    state: Arc<AppState>,
    section: SchoolSectionModelNew,
) -> DbClassResult<SchoolSectionModelGet> {
    let index = IndexModel::builder()
        .keys(doc! {"name" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.school_section.collection.create_index(index).await {
        return Err(DbClassError::OtherError { err: e.to_string() });
    }

    let get = state
        .db
        .school_section
        .collection
        .find_one(doc! {"name" : section.name.clone()})
        .await
        .map_err(|e| DbClassError::OtherError {
            err: format!(
                "Some thing went wrong to get school section error is : 😡 [{}] 😡",
                e
            ),
        })?;

    if let Some(r) = get {
        return Err(DbClassError::OtherError {
            err: format!("School Section name already exists [{}]", r.name),
        });
    }

    let create = state
        .db
        .school_section
        .create(
            SchoolSectionModel::new(section),
            Some("School section".to_string()),
        )
        .await?;
    let get = state
        .db
        .school_section
        .get_one_by_id(create, Some("School section".to_string()))
        .await?;
    Ok(SchoolSectionModel::format(get))
}

pub async fn get_all_school_section(
    state: Arc<AppState>,
) -> DbClassResult<Vec<SchoolSectionModelGet>> {
    let get_all = state
        .db
        .school_section
        .get_many(None, Some("School section".to_string()))
        .await?;
    Ok(get_all
        .into_iter()
        .map(SchoolSectionModel::format)
        .collect())
}

pub async fn get_school_section_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SchoolSectionModelGet> {
    let get = state
        .db
        .school_section
        .get_one_by_id(id, Some("School section".to_string()))
        .await?;

    Ok(SchoolSectionModel::format(get))
}

pub async fn update_school_section_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    section: SchoolSectionModelPut,
) -> DbClassResult<SchoolSectionModelGet> {
    let put = state
        .db
        .school_section
        .update(
            id,
            SchoolSectionModel::put(section),
            Some("School Section".to_string()),
        )
        .await?;
    Ok(SchoolSectionModel::format(put))
}

pub async fn delete_school_section_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<SchoolSectionModelGet> {
    let put = state
        .db
        .school_section
        .delete(id, Some("School Section".to_string()))
        .await?;
    Ok(SchoolSectionModel::format(put))
}
