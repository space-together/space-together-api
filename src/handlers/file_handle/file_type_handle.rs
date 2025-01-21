use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::file_controller::file_type_controller::{
        create_file_type, delete_file_type_by_id, get_all_file_type, get_file_type_by_id,
        get_file_type_by_username, update_file_type_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        file_model::file_type_model::{FileTypeModelNew, FileTypeModelPut},
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn create_file_type_handle(
    state: Data<AppState>,
    file_type: Json<FileTypeModelNew>,
) -> impl Responder {
    let create = create_file_type(state.into_inner(), file_type.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_file_type_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_file_type(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_file_type_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_file_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn get_file_type_by_username_handle(
    state: Data<AppState>,
    username: Path<String>,
) -> impl Responder {
    match get_file_type_by_username(state.into_inner(), username.into_inner()).await {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn delete_file_type_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_file_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_file_type_by_id_handle(
    state: Data<AppState>,
    file_type: Json<FileTypeModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => {
            match update_file_type_by_id(state.into_inner(), i, file_type.into_inner()).await {
                Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: e.to_string(),
                }),
                Ok(r) => HttpResponse::Ok().json(r),
            }
        }
    }
}
