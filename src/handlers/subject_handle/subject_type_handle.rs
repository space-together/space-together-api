use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::subject_controller::subject_type_controller::{
        create_subject_type, delete_subject_type_by_id, get_all_subject_type,
        get_subject_type_by_id, update_subject_type_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        request_error_model::ReqErrModel,
        subject_model::subject_type_model::{SubjectTypeModelNew, SubjectTypeModelPut},
    },
    AppState,
};

pub async fn create_subject_type_handle(
    state: Data<AppState>,
    subject_type: Json<SubjectTypeModelNew>,
) -> impl Responder {
    let create = create_subject_type(state.into_inner(), subject_type.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_subject_type_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_subject_type(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_subject_type_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_subject_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_subject_type_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_subject_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_subject_type_by_id_handle(
    state: Data<AppState>,
    subject_type: Json<SubjectTypeModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => {
            match update_subject_type_by_id(state.into_inner(), i, subject_type.into_inner()).await
            {
                Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: e.to_string(),
                }),
                Ok(r) => HttpResponse::Ok().json(r),
            }
        }
    }
}
