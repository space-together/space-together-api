use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_err::{UserError, UserResult},
    libs::{
        classes::db_crud::GetManyByField,
        functions::object_id::change_insertoneresult_into_object_id,
    },
    models::{
        images_model::profile_images_model::{
            ProfileImageModel, ProfileImageModelGet, ProfileImageModelNew,
        },
        user_model::user_model_model::{
            UserModelGet, UserModelNew, UserModelPut, UsersUpdateManyModel,
        },
    },
    AppState,
};

use super::user_images_controller::get_user_images;

pub async fn controller_create_user(
    user: UserModelNew,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    if ObjectId::from_str(&user.role).is_err() {
        return Err(UserError::InvalidUserRoleId);
    }

    match state
        .db
        .user_role
        .get_user_role_by_id(ObjectId::from_str(&user.role).unwrap())
        .await
    {
        Err(_) => Err(UserError::InvalidId),
        Ok(role) => match state.db.user.create_user(user).await {
            Err(err) => Err(err),
            Ok(insert_id) => match state
                .db
                .user
                .get_user_by_id(change_insertoneresult_into_object_id(insert_id))
                .await
            {
                Err(err) => Err(err),
                Ok(res) => {
                    let mut user_get = UserModelGet::format(res);
                    user_get.role = role.role;
                    Ok(user_get)
                }
            },
        },
    }
}

pub async fn controller_user_update_by_id(
    user: UserModelPut,
    id: ObjectId,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    if let Some(role) = user.role.clone() {
        if state
            .db
            .user_role
            .get_user_role_by_rl(role.clone())
            .await
            .is_err()
        {
            return Err(UserError::InvalidUserRoleId);
        }
    }

    let mut user_images = get_user_images(&state, &id).await?;

    if let Some(image) = user.image.clone() {
        let avatar = ProfileImageModelNew {
            src: image,
            user_id: id,
        };

        let avatar_new = state
            .db
            .avatars
            .create(ProfileImageModel::new(avatar), Some("Avatar".to_string()))
            .await;

        match avatar_new {
            Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
            Ok(i) => match state
                .db
                .avatars
                .get_one_by_id(i, Some("Avatar".to_string()))
                .await
            {
                Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                Ok(image) => user_images.push(ProfileImageModel::format(image)),
            },
        }
    }

    match state.db.user.update_user_by_id(user, id).await {
        Err(err) => Err(err),
        Ok(doc) => match state.db.user.get_user_by_id(doc.id.unwrap()).await {
            Ok(u) => {
                let mut my_user = UserModelGet::format(u);
                my_user.image = Some(user_images);

                Ok(my_user)
            }
            Err(err) => Err(err),
        },
    }
}

pub async fn controller_user_update_by_username(
    user: UserModelPut,
    username: String,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    if let Some(role) = user.role.clone() {
        if state
            .db
            .user_role
            .get_user_role_by_rl(role.clone())
            .await
            .is_err()
        {
            return Err(UserError::InvalidUserRoleId);
        }
    }

    let image_collection = Some("Avatar".to_string());

    let get_user_user = state.db.user.get_user_by_username(username.clone()).await;

    let id = get_user_user?.id.unwrap();

    let mut user_images: Vec<ProfileImageModelGet> = match state
        .db
        .avatars
        .get_many(
            Some(GetManyByField {
                value: id,
                field: "user_id".to_string(),
            }),
            image_collection.clone(),
        )
        .await
    {
        Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
        Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
    };

    if let Some(image) = user.image.clone() {
        let avatar = ProfileImageModelNew {
            src: image,
            user_id: id,
        };

        let avatar_new = state
            .db
            .avatars
            .create(ProfileImageModel::new(avatar), image_collection.clone())
            .await;

        match avatar_new {
            Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
            Ok(i) => match state.db.avatars.get_one_by_id(i, image_collection).await {
                Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                Ok(image) => user_images.push(ProfileImageModel::format(image)),
            },
        }
    }

    match state.db.user.update_user_by_username(user, username).await {
        Err(err) => Err(err),
        Ok(doc) => match state.db.user.get_user_by_id(doc.id.unwrap()).await {
            Ok(u) => {
                let mut my_user = UserModelGet::format(u);
                my_user.image = Some(user_images);

                Ok(my_user)
            }
            Err(err) => Err(err),
        },
    }
}

pub async fn controller_get_user_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> UserResult<UserModelGet> {
    match state.db.user.get_user_by_id(id).await {
        Ok(res) => {
            let role = state
                .db
                .user_role
                .get_user_role_by_id(res.role.unwrap())
                .await;
            match role {
                Ok(role) => {
                    let user_images: Vec<ProfileImageModelGet> = match state
                        .db
                        .avatars
                        .get_many(
                            Some(GetManyByField {
                                value: id,
                                field: "user_id".to_string(),
                            }),
                            Some("Avatar".to_string()),
                        )
                        .await
                    {
                        Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                        Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
                    };

                    let mut user_get = UserModelGet::format(res);
                    user_get.role = role.role;
                    user_get.image = Some(user_images);
                    Ok(user_get)
                }
                Err(err) => Err(UserError::CanNotGetRole {
                    error: err.to_string(),
                }),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_get_by_username(
    state: Arc<AppState>,
    username: String,
) -> UserResult<UserModelGet> {
    match state.db.user.get_user_by_username(username).await {
        Ok(res) => {
            let role = state
                .db
                .user_role
                .get_user_role_by_id(res.role.unwrap())
                .await;
            match role {
                Ok(role) => {
                    let user_images: Vec<ProfileImageModelGet> = match state
                        .db
                        .avatars
                        .get_many(
                            Some(GetManyByField {
                                value: res.id.unwrap(),
                                field: "user_id".to_string(),
                            }),
                            Some("Avatar".to_string()),
                        )
                        .await
                    {
                        Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                        Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
                    };

                    let mut user_get = UserModelGet::format(res);
                    user_get.role = role.role;
                    user_get.image = Some(user_images);
                    Ok(user_get)
                }
                Err(err) => Err(UserError::CanNotGetRole {
                    error: err.to_string(),
                }),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_get_user_by_email(
    state: Arc<AppState>,
    email: String,
) -> UserResult<UserModelGet> {
    match state.db.user.get_user_by_email(email).await {
        Ok(res) => {
            let role = state
                .db
                .user_role
                .get_user_role_by_id(res.role.unwrap())
                .await;
            match role {
                Ok(role) => {
                    let user_images: Vec<ProfileImageModelGet> = match state
                        .db
                        .avatars
                        .get_many(
                            Some(GetManyByField {
                                value: res.id.unwrap(),
                                field: "user_id".to_string(),
                            }),
                            Some("Avatar".to_string()),
                        )
                        .await
                    {
                        Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                        Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
                    };

                    let mut user_get = UserModelGet::format(res);
                    user_get.role = role.role;
                    user_get.image = Some(user_images);
                    Ok(user_get)
                }
                Err(err) => Err(UserError::CanNotGetRole {
                    error: err.to_string(),
                }),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_by_id(
    id: ObjectId,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    match state.db.user.delete_user_by_id(id).await {
        Ok(res) => {
            let user_images: Vec<ProfileImageModelGet> = match state
                .db
                .avatars
                .get_many(
                    Some(GetManyByField {
                        value: id,
                        field: "user_id".to_string(),
                    }),
                    Some("Avatar".to_string()),
                )
                .await
            {
                Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
            };

            for image in user_images {
                let delete_image = state
                    .db
                    .avatars
                    .delete(
                        ObjectId::from_str(&image.id).unwrap(),
                        Some("Avatar".to_string()),
                    )
                    .await;

                if let Err(e) = delete_image {
                    return Err(UserError::SomeError { err: e.to_string() });
                }
            }

            let user_get = UserModelGet::format(res);
            Ok(user_get)
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_many(
    users: Vec<ObjectId>,
    state: Arc<AppState>,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user.delete_users(users).await {
        Ok(res) => {
            for user in res.clone() {
                let user_images: Vec<ProfileImageModelGet> = match state
                    .db
                    .avatars
                    .get_many(
                        Some(GetManyByField {
                            value: ObjectId::from_str(&user.id).unwrap(),
                            field: "user_id".to_string(),
                        }),
                        Some("Avatar".to_string()),
                    )
                    .await
                {
                    Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                    Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
                };

                for image in user_images {
                    let delete_image = state
                        .db
                        .avatars
                        .delete(
                            ObjectId::from_str(&image.id).unwrap(),
                            Some("Avatar".to_string()),
                        )
                        .await;

                    if let Err(e) = delete_image {
                        return Err(UserError::SomeError { err: e.to_string() });
                    }
                }
            }
            Ok(res)
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_update_many(
    users: Vec<UsersUpdateManyModel>,
    state: Arc<AppState>,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user.update_many(users).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_by_username(
    state: Arc<AppState>,
    username: String,
) -> UserResult<UserModelGet> {
    match state.db.user.delete_user_by_username(username).await {
        Ok(res) => {
            let user_images: Vec<ProfileImageModelGet> = match state
                .db
                .avatars
                .get_many(
                    Some(GetManyByField {
                        value: res.id.unwrap(),
                        field: "user_id".to_string(),
                    }),
                    Some("Avatar".to_string()),
                )
                .await
            {
                Err(e) => return Err(UserError::SomeError { err: e.to_string() }),
                Ok(images) => images.into_iter().map(ProfileImageModel::format).collect(),
            };

            for image in user_images {
                let delete_image = state
                    .db
                    .avatars
                    .delete(
                        ObjectId::from_str(&image.id).unwrap(),
                        Some("Avatar".to_string()),
                    )
                    .await;

                if let Err(e) = delete_image {
                    return Err(UserError::SomeError { err: e.to_string() });
                }
            }
            Ok(UserModelGet::format(res))
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_users(state: Arc<AppState>) -> UserResult<Vec<UserModelGet>> {
    let get_all = state.db.user.get_all_users().await;
    match get_all {
        Ok(res) => {
            let mut users: Vec<UserModelGet> = Vec::new();

            for mut user in res {
                if let Ok(role) = state
                    .db
                    .user_role
                    .get_user_role_by_id(ObjectId::from_str(&user.role).unwrap())
                    .await
                {
                    let mut user_get = user.clone();

                    let user_images =
                        get_user_images(&state, &ObjectId::from_str(&user.id).unwrap()).await?;

                    user_get.image = Some(user_images);
                    user_get.role = role.role;
                    users.push(user_get);
                } else {
                    let user_images =
                        get_user_images(&state, &ObjectId::from_str(&user.id).unwrap()).await?;

                    user.image = Some(user_images);
                    users.push(user.clone());
                }
            }
            Ok(users)
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_users_get_all_by_role(
    state: Arc<AppState>,
    role: String,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user_role.get_user_role_by_rl(role).await {
        Err(err) => Err(UserError::CanNotGetRole {
            error: err.to_string(),
        }),
        Ok(res) => match state.db.user.get_users_by_rl(res.id.unwrap()).await {
            Ok(res) => {
                let mut users: Vec<UserModelGet> = Vec::new();

                for mut user in res {
                    if let Ok(role) = state
                        .db
                        .user_role
                        .get_user_role_by_id(ObjectId::from_str(&user.role).unwrap())
                        .await
                    {
                        let mut user_get = user.clone();

                        let user_images =
                            get_user_images(&state, &ObjectId::from_str(&user.id).unwrap()).await?;

                        user_get.image = Some(user_images);
                        user_get.role = role.role;
                        users.push(user_get);
                    } else {
                        let user_images =
                            get_user_images(&state, &ObjectId::from_str(&user.id).unwrap()).await?;

                        user.image = Some(user_images);
                        users.push(user.clone());
                    }
                }
                Ok(users)
            }
            Err(err) => Err(err),
        },
    }
}
