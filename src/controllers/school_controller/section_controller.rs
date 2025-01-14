use std::sync::Arc;

use crate::{
    error::db_class_error::DbClassResult,
    models::school_model::school_section_model::{
        SchoolSectionModel, SchoolSectionModelGet, SchoolSectionModelNew,
    },
    AppState,
};

pub async fn create_school_section(
    state: Arc<AppState>,
    section: SchoolSectionModelNew,
) -> DbClassResult<SchoolSectionModelGet> {
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
