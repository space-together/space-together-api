use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::classes::db_crud::GetManyByField,
    models::images_model::school_logo_model::SchoolLogoModel,
    AppState,
};

pub async fn fetch_school_logo(
    state: &Arc<AppState>,
    school_id: &str,
    collection: Option<String>,
) -> DbClassResult<Option<SchoolLogoModel>> {
    let field = "school_id".to_string();
    let value = ObjectId::from_str(school_id)
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

    match state
        .db
        .school_logo
        .get_one_by_field(GetManyByField { field, value }, collection)
        .await
    {
        Ok(logo) => Ok(Some(logo)),
        Err(_) => Ok(None),
    }
}
