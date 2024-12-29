use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_err::{UserError, UserResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
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

pub async fn controller_create_user(
    user: UserModelNew,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    if ObjectId::from_str(&user.rl).is_err() {
        return Err(UserError::InvalidUserRoleId);
    }

    match state
        .db
        .user_role
        .get_user_role_by_id(ObjectId::from_str(&user.rl).unwrap())
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
                    user_get.rl = role.rl;
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
    if let Some(role) = user.rl.clone() {
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

    let mut user_images: Vec<ProfileImageModelGet> =
        state.db.avatars.get_many(id.clone(), "Avatar").await;

    if let Some(image) = user.im.clone() {
        let avatar = ProfileImageModelNew { src: image, ui: id };

        let avatar_new = state
            .db
            .avatars
            .create(ProfileImageModel::new(avatar), Some("avatar".to_string()))
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
                my_user.im = Some(user_images);

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
    match state.db.user.update_user_by_username(user, username).await {
        Err(err) => Err(err),
        Ok(doc) => match state.db.user.get_user_by_id(doc.id.unwrap()).await {
            Ok(res) => Ok(UserModelGet::format(res)),
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
                .get_user_role_by_id(res.rl.unwrap())
                .await;

            match role {
                Ok(role) => {
                    let mut user_get = UserModelGet::format(res);
                    user_get.rl = role.rl;
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
        Ok(res) => Ok(UserModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_by_id(
    id: ObjectId,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    match state.db.user.delete_user_by_id(id).await {
        Ok(res) => Ok(UserModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_many(
    users: Vec<ObjectId>,
    state: Arc<AppState>,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user.delete_users(users).await {
        Ok(res) => Ok(res),
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
        Ok(res) => Ok(UserModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_users(state: Arc<AppState>) -> UserResult<Vec<UserModelGet>> {
    let get_all = state.db.user.get_all_users().await;
    match get_all {
        Ok(res) => {
            let mut users: Vec<UserModelGet> = Vec::new();
            for user in res.iter() {
                if let Ok(role) = state
                    .db
                    .user_role
                    .get_user_role_by_id(ObjectId::from_str(&user.rl).unwrap())
                    .await
                {
                    let mut user_get = (*user).clone();
                    user_get.rl = role.rl;
                    users.push(user_get);
                } else {
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
            Ok(res) => Ok(res.into_iter().map(UserModelGet::format).collect()),
            Err(err) => Err(err),
        },
    }
}
