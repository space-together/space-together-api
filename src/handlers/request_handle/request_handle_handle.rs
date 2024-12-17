use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::request_controller::request_controller_controller::{
        controller_request_create, controller_request_delete, controller_request_get_all,
        controller_request_get_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        request_error_model::ReqErrModel, request_model::request_model_model::RequestModelNew,
    },
    AppState,
};

pub async fn handle_request_create(
    state: Data<AppState>,
    request: Json<RequestModelNew>,
) -> impl Responder {
    if let Err(err) = change_string_into_object_id(request.rl.clone()) {
        return HttpResponse::BadRequest().json(err);
    }
    match controller_request_create(state.into_inner(), request.into_inner()).await {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(res) => HttpResponse::Created().json(res),
    }
}

pub async fn handle_request_get_all(state: Data<AppState>) -> impl Responder {
    match controller_request_get_all(state.into_inner()).await {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(res) => HttpResponse::Created().json(res),
    }
}

pub async fn handle_request_get_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match controller_request_get_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(res) => HttpResponse::Created().json(res),
        },
    }
}

pub async fn handle_request_delete(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match controller_request_delete(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(res) => HttpResponse::Created().json(res),
        },
    }
}
