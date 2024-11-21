use crate::{
    controllers::user_controller::user_role_controller::{
        controller_create_user_model, controller_get_all_user_roles, controller_get_user_role,
        controller_get_user_role_name, controller_user_role_delete, controller_user_role_update,
    },
    models::{request_error_model::ReqErrModel, user_model::user_role_model::UserRoleModelNew},
    AppState,
};
use actix_web::{
    web::{self, Data, Json, Query},
    HttpResponse, Responder,
};
use serde::Deserialize;

pub async fn handle_create_user_role(
    state: web::Data<AppState>,
    role: web::Json<UserRoleModelNew>,
) -> impl Responder {
    let create = controller_create_user_model(role.into_inner(), state.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}

pub async fn handle_get_user_role(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> impl Responder {
    let get = controller_get_user_role(id.into_inner(), state.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub name: String,
}

pub async fn handle_user_role_get_by_name(
    state: Data<AppState>,
    query: Query<SearchParams>,
) -> impl Responder {
    match controller_get_user_role_name(query.name.clone(), state.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}

pub async fn handle_user_role_delete(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> impl Responder {
    match controller_user_role_delete(id.into_inner(), state.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}

pub async fn handle_user_role_update(
    state: web::Data<AppState>,
    role: Json<UserRoleModelNew>,
    id: web::Path<String>,
) -> impl Responder {
    match controller_user_role_update(id.into_inner(), role.into_inner(), state.into_inner()).await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}

pub async fn handle_get_all_user_roles(state: web::Data<AppState>) -> impl Responder {
    let get = controller_get_all_user_roles(state.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => {
            let error = ReqErrModel {
                message: err.to_string(),
            };
            HttpResponse::BadRequest().json(error)
        }
    }
}
