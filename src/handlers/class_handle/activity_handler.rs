use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::activity_controller::{
        controller_activity_create, controller_activity_get_by_class,
        controller_activity_get_by_group, controller_activity_get_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{class_model::activity_model::ActivityModelNew, request_error_model::ReqErrModel},
    AppState,
};

pub async fn handle_activity_get_by_class(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Ok(obj) => match controller_activity_get_by_class(state.into_inner(), obj).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub async fn handle_activity_get_by_group(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Ok(obj) => match controller_activity_get_by_group(state.into_inner(), obj).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub async fn handle_activity_get_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Ok(obj) => match controller_activity_get_by_id(state.into_inner(), obj).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub async fn handle_activity_create(
    state: Data<AppState>,
    data: Json<ActivityModelNew>,
) -> impl Responder {
    match controller_activity_create(state.into_inner(), data.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
