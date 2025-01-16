use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::education_controller::education_controller_controller::{
        create_education, delete_education_by_id, get_all_education, get_education_by_id,
        update_education_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        education_model::education_model_model::{EducationModelNew, EducationModelPut},
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn create_education_handle(
    state: Data<AppState>,
    education: Json<EducationModelNew>,
) -> impl Responder {
    let create = create_education(state.into_inner(), education.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_education_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_education(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_education_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_education_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_education_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_education_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_education_by_id_handle(
    state: Data<AppState>,
    education: Json<EducationModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => {
            match update_education_by_id(state.into_inner(), i, education.into_inner()).await {
                Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: e.to_string(),
                }),
                Ok(r) => HttpResponse::Ok().json(r),
            }
        }
    }
}
