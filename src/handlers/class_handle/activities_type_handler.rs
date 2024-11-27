use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::activities_type_controller::{
        controller_activities_type_create, controller_activities_type_delete_by_id,
        controller_activities_type_get_all, controller_activities_type_get_by_id,
        controller_activities_type_update_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        class_model::activities_type_model::{ActivitiesTypeModelNew, ActivitiesTypeModelPut},
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
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => match controller_activities_type_get_by_id(state.into_inner(), i).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}
pub async fn handle_activity_type_delete_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => match controller_activities_type_delete_by_id(state.into_inner(), i).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_activity_type_update_by_id(
    state: Data<AppState>,
    activity_type: Json<ActivitiesTypeModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => match controller_activities_type_update_by_id(
            state.into_inner(),
            i,
            activity_type.into_inner(),
        )
        .await
        {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
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
