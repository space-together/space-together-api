use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::school_controller::school_controller_controller::{
        controller_school_create, controller_school_get, controller_school_get_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{request_error_model::ReqErrModel, school_model::school_model_model::SchoolModelNew},
    AppState,
};

pub async fn handle_school_create(
    state: Data<AppState>,
    school: Json<SchoolModelNew>,
) -> impl Responder {
    let create = controller_school_create(state.into_inner(), school.into_inner()).await;
    match create {
        Ok(r) => HttpResponse::Created().json(r),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
    }
}

pub async fn handle_school_get(state: Data<AppState>) -> impl Responder {
    let get = controller_school_get(state.into_inner()).await;
    match get {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
    }
}

pub async fn handle_school_get_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match controller_school_get_by_id(state.into_inner(), i).await {
            Ok(r) => HttpResponse::Ok().json(r),
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
        },
    }
}
