use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::activities_type_controller::{
        controller_activities_type_create, controller_activities_type_get_all,
        controller_activities_type_get_by_id,
    },
    models::{
        class_model::activities_type_model::ActivitiesTypeModelNew,
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_activity_type_create(
    state: Data<AppState>,
    data: Json<ActivitiesTypeModelNew>,
) -> impl Responder {
    let create = controller_activities_type_create(state.into_inner(), data.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_activity_type_get_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    let get = controller_activities_type_get_by_id(state.into_inner(), id.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_activity_type_get_all(state: Data<AppState>) -> impl Responder {
    let get_all = controller_activities_type_get_all(state.into_inner()).await;
    match get_all {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
