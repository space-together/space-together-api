use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};

use crate::{
    libs::auth::{
        oauth2::oauth2::{fetch_oauth_token, oauth2_provider_url, FetchOauthTokenModel},
        user_register::{user_login, user_register},
        user_session::{
            create_user_session, delete_user_session, get_user_session, update_user_session_expires,
        },
    },
    models::{
        request_error_model::ReqErrModel,
        user_model::user_model_model::{AccountProviders, UserLoginModel, UserModel, UserModelNew},
    },
    AppState,
};

pub async fn user_register_router(
    data: Json<UserModelNew>,
    state: Data<AppState>,
) -> impl Responder {
    match user_register(state.into_inner(), data.into_inner(), &None).await {
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

pub async fn update_user_session_expires_router(
    req: HttpRequest,
    state: Data<AppState>,
) -> impl Responder {
    if let Some(token) = req.headers().get("Authorization") {
        match update_user_session_expires(token.to_str().unwrap(), &state.into_inner()).await {
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

pub async fn oauth2_provider_url_router(
    state: Data<AppState>,
    provider: Path<AccountProviders>,
) -> impl Responder {
    match oauth2_provider_url(&state.into_inner(), provider.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
    }
}

pub async fn fetch_oauth_token_router(
    state: Data<AppState>,
    token: Json<FetchOauthTokenModel>,
) -> impl Responder {
    match fetch_oauth_token(&state.into_inner(), token.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
    }
}

// user session
pub async fn create_user_session_router(
    state: Data<AppState>,
    user: Json<UserModel>,
) -> impl Responder {
    match create_user_session(user.into_inner(), state.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e }),
    }
}
