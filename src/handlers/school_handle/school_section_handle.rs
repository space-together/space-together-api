use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::school_controller::school_section_controller::{
        create_school_section, delete_school_section_by_id, get_all_school_section,
        get_school_section_by_id, update_school_section_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        request_error_model::ReqErrModel,
        school_model::school_section_model::{SchoolSectionModelNew, SchoolSectionModelPut},
    },
    AppState,
};

pub async fn create_school_section_handle(
    state: Data<AppState>,
    section: Json<SchoolSectionModelNew>,
) -> impl Responder {
    let create = create_school_section(state.into_inner(), section.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn update_school_section_handle(
    state: Data<AppState>,
    id: Path<String>,
    section: Json<SchoolSectionModelPut>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => {
            match update_school_section_by_id(state.into_inner(), i, section.into_inner()).await {
                Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: e.to_string(),
                }),
                Ok(r) => HttpResponse::Created().json(r),
            }
        }
    }
}

pub async fn get_school_section_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_school_section_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Created().json(r),
        },
    }
}

pub async fn delete_school_section_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_school_section_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Created().json(r),
        },
    }
}

pub async fn get_all_school_section_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_school_section(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}
