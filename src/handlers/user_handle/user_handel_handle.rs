use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::user_controller::user_controller_controller::{
        controller_create_user, controller_get_all_users, controller_get_user_by_id,
    },
    models::{request_error_model::ReqErrModel, user_model::user_model_model::UserModelNew},
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
    let get = controller_get_user_by_id(state.into_inner(), id.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
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
