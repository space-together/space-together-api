use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::user_controller::user_controller_controller::{
        controller_create_user, controller_get_all_users, controller_get_user_by_id,
        controller_user_delete_by_id, controller_user_delete_by_username,
        controller_user_get_by_username, controller_user_update_by_id,
        controller_user_update_by_username, controller_users_get_all_by_role,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        request_error_model::ReqErrModel,
        user_model::user_model_model::{UserModelNew, UserModelPut},
    },
    AppState,
};

pub async fn handle_create_user(state: Data<AppState>, user: Json<UserModelNew>) -> impl Responder {
    let create = controller_create_user(user.into_inner(), state.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_get_user_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_get_user_by_id(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_get_user_by_username(
    state: Data<AppState>,
    username: Path<String>,
) -> impl Responder {
    match controller_user_get_by_username(state.into_inner(), username.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_user_update_by_id(
    state: Data<AppState>,
    id: Path<String>,
    user: Json<UserModelPut>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => {
            match controller_user_update_by_id(user.into_inner(), _id, state.into_inner()).await {
                Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: err.to_string(),
                }),
                Ok(data) => HttpResponse::Ok().json(data),
            }
        }
    }
}

pub async fn handle_user_delete_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_user_delete_by_id(_id, state.into_inner()).await {
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
            Ok(data) => HttpResponse::Ok().json(data),
        },
    }
}

pub async fn handle_user_update_by_username(
    state: Data<AppState>,
    username: Path<String>,
    user: Json<UserModelPut>,
) -> impl Responder {
    match controller_user_update_by_username(
        user.into_inner(),
        username.into_inner(),
        state.into_inner(),
    )
    .await
    {
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
        Ok(data) => HttpResponse::Ok().json(data),
    }
}

pub async fn handle_user_delete_by_username(
    state: Data<AppState>,
    username: Path<String>,
) -> impl Responder {
    match controller_user_delete_by_username(state.into_inner(), username.into_inner()).await {
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
        Ok(data) => HttpResponse::Ok().json(data),
    }
}

pub async fn handle_get_all_users(state: Data<AppState>) -> impl Responder {
    let get = controller_get_all_users(state.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
pub async fn handle_user_get_all_by_role(
    state: Data<AppState>,
    role: Path<String>,
) -> impl Responder {
    match controller_users_get_all_by_role(state.into_inner(), role.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
