use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::class_room_type_controller::{
        create_class_room_type, delete_class_room_type_by_id, get_all_class_room_type,
        get_class_room_type_by_id, update_class_room_type_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        class_model::class_room_type_model::{ClassRoomTypeModelNew, ClassRoomTypeModelPut},
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn create_class_room_type_handle(
    state: Data<AppState>,
    class_room_type: Json<ClassRoomTypeModelNew>,
) -> impl Responder {
    let create = create_class_room_type(state.into_inner(), class_room_type.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_class_room_type_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_class_room_type(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_class_room_type_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_class_room_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_class_room_type_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_class_room_type_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_class_room_type_by_id_handle(
    state: Data<AppState>,
    class_room_type: Json<ClassRoomTypeModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => {
            match update_class_room_type_by_id(state.into_inner(), i, class_room_type.into_inner())
                .await
            {
                Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: e.to_string(),
                }),
                Ok(r) => HttpResponse::Ok().json(r),
            }
        }
    }
}
