use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::class_group_controller::{
        controller_class_group_create, controller_class_group_get_all,
        controller_get_class_group_by_id,
    },
    models::{
        class_model::class_group_model::class_group_model_model::ClassGroupModelNew,
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_create_class_groups(
    state: Data<AppState>,
    group: Json<ClassGroupModelNew>,
) -> impl Responder {
    let create = controller_class_group_create(state.into_inner(), group.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_class_group_get_all(state: Data<AppState>) -> impl Responder {
    let all = controller_class_group_get_all(state.into_inner()).await;
    match all {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_get_class_group_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    let get = controller_get_class_group_by_id(state.into_inner(), id.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
