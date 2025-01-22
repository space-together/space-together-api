use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::file_controller::file_controller_controller::{
        create_file, delete_file_by_id, get_all_file, get_file_by_id, update_file_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        file_model::file_model_model::{FileModelNew, FileModelPut},
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn create_file_handle(state: Data<AppState>, file: Json<FileModelNew>) -> impl Responder {
    let create = create_file(state.into_inner(), file.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_file_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_file(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_file_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_file_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_file_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_file_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_file_by_id_handle(
    state: Data<AppState>,
    file: Json<FileModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match update_file_by_id(state.into_inner(), i, file.into_inner()).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}
