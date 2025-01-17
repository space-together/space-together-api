use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::school_controller::sector_controller::{
        create_sector, delete_sector_by_id, get_all_sector, get_sector_by_id,
        get_sector_by_username, update_sector_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        request_error_model::ReqErrModel,
        school_model::sector_model::{SectorModelNew, SectorModelPut},
    },
    AppState,
};

pub async fn create_sector_handle(
    state: Data<AppState>,
    sector: Json<SectorModelNew>,
) -> impl Responder {
    let create = create_sector(state.into_inner(), sector.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_sector_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_sector(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_sector_by_username_handle(
    state: Data<AppState>,
    username: Path<String>,
) -> impl Responder {
    let get = get_sector_by_username(state.into_inner(), username.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_sector_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_sector_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_sector_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_sector_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_sector_by_id_handle(
    state: Data<AppState>,
    sector: Json<SectorModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match update_sector_by_id(state.into_inner(), i, sector.into_inner()).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}
