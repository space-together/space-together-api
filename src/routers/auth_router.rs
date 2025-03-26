use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};

use crate::{
    libs::auth::{
        user_register::{user_login, user_register},
        user_session::{delete_user_session, get_user_session},
    },
    models::{
        request_error_model::ReqErrModel,
        user_model::user_model_model::{UserLoginModel, UserModelNew},
    },
    AppState,
};

pub async fn user_register_router(
    data: Json<UserModelNew>,
    state: Data<AppState>,
) -> impl Responder {
    match user_register(state.into_inner(), data.into_inner()).await {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
        Ok(d) => HttpResponse::Created().json(d),
    }
}

pub async fn user_login_router(
    data: Json<UserLoginModel>,
    state: Data<AppState>,
) -> impl Responder {
    match user_login(state.into_inner(), data.into_inner()).await {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
        Ok(d) => HttpResponse::Ok().json(d),
    }
}

pub async fn get_user_session_router(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    if let Some(token) = req.headers().get("Authorization") {
        match get_user_session(token.to_str().unwrap(), state.into_inner()).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
            Ok(d) => HttpResponse::Ok().json(d),
        }
    } else {
        HttpResponse::BadRequest().json(ReqErrModel {
            message: "Token not found".to_string(),
        })
    }
}

pub async fn delete_user_session_router(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    if let Some(token) = req.headers().get("Authorization") {
        match delete_user_session(token.to_str().unwrap(), state.into_inner()).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
            Ok(d) => HttpResponse::Ok().json(d),
        }
    } else {
        HttpResponse::BadRequest().json(ReqErrModel {
            message: "Token not found".to_string(),
        })
    }
}
