use crate::{
    controllers::user_controller::user_role_controller::{
        controller_create_user_model, controller_get_all_user_roles, controller_get_user_role,
    },
    models::{request_error_model::ReqErrModel, user_model::user_role_model::UserRoleModelNew},
    AppState,
};
use actix_web::{web, HttpResponse, Responder};

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
