use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_err::{UserError, UserResult},
    libs::classes::db_crud::GetManyByField,
    models::images_model::profile_images_model::{
        ProfileImageModel, ProfileImageModelGet, ProfileImageModelNew,
    },
    AppState,
};

pub async fn fetch_user_images(
    state: &AppState,
    user_id: &ObjectId,
) -> UserResult<Vec<ProfileImageModelGet>> {
    state
        .db
        .avatars
        .get_many(
            Some(GetManyByField {
                value: *user_id,
                field: "user_id".to_string(),
            }),
            Some("Avatar".to_string()),
        )
        .await
        .map(|images| images.into_iter().map(ProfileImageModel::format).collect())
        .map_err(|e| UserError::SomeError { err: e.to_string() })
}

/// Handles user avatar creation or updates
pub async fn process_user_avatar(
    state: &AppState,
    user_id: &ObjectId,
    avatar_src: Option<String>,
) -> UserResult<Vec<ProfileImageModelGet>> {
    let mut user_images = fetch_user_images(state, user_id).await?;
    if let Some(image) = avatar_src {
        let avatar = ProfileImageModelNew {
            src: image,
            user_id: *user_id,
        };
        let avatar_id = state
            .db
            .avatars
            .create(ProfileImageModel::new(avatar), Some("Avatar".to_string()))
            .await
            .map_err(|e| UserError::SomeError { err: e.to_string() })?;

        let avatar_image = state
            .db
            .avatars
            .get_one_by_id(avatar_id, Some("Avatar".to_string()))
            .await
            .map_err(|e| UserError::SomeError { err: e.to_string() })?;
        user_images.push(ProfileImageModel::format(avatar_image));
    }
    Ok(user_images)
}
