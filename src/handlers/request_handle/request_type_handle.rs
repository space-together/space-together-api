use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::{
    controllers::request_controller::request_type_controller::{
        controllers_request_type_create, controllers_request_type_get_all,
    },
    models::{
        request_error_model::ReqErrModel, request_model::request_type_model::RequestTypeModelNew,
    },
    AppState,
};

pub async fn handle_request_type_create(
    state: Data<AppState>,
    role: Json<RequestTypeModelNew>,
) -> impl Responder {
    match controllers_request_type_create(state.into_inner(), role.into_inner()).await {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_request_type_get_all(state: Data<AppState>) -> impl Responder {
    match controllers_request_type_get_all(state.into_inner()).await {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
