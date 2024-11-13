use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::class_controller_controller::{
        controller_create_class, controller_get_all_classes, controller_get_class_by_id,
    },
    models::{class_model::class_model_model::ClassModelNew, request_error_model::ReqErrModel},
    AppState,
};

pub async fn handle_create_class(
    state: Data<AppState>,
    class: Json<ClassModelNew>,
) -> impl Responder {
    let create = controller_create_class(state.into_inner(), class.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_get_class_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    let get = controller_get_class_by_id(state.into_inner(), id.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handler_get_all_classes(state: Data<AppState>) -> impl Responder {
    let all = controller_get_all_classes(state.into_inner()).await;
    match all {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
