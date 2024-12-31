use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_err::UserError,
    libs::classes::db_crud::GetManyByField,
    models::images_model::profile_images_model::{ProfileImageModel, ProfileImageModelGet},
    AppState,
};

pub async fn get_user_images(
    state: &Arc<AppState>,
    user_id: &ObjectId,
) -> Result<Vec<ProfileImageModelGet>, UserError> {
    let images_result = state
        .db
        .avatars
        .get_many(
            Some(GetManyByField {
                value: *user_id,
                field: "ui".to_string(),
            }),
            Some("Avatar".to_string()),
        )
        .await;

    match images_result {
        Ok(mut images) => {
            images.sort_by(|a, b| b.co.cmp(&a.co)); // Sorting LIFO
            Ok(images.into_iter().map(ProfileImageModel::format).collect())
        }
        Err(e) => Err(UserError::SomeError { err: e.to_string() }),
    }
}
