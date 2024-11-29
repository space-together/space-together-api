use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::conversation_controller::message_controller::{
        controller_message_create, controller_message_delete_by_id,
        controller_message_get_all_by_conversation,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        conversation_model::message_model::MessageModelNew, request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_message_create(
    state: Data<AppState>,
    message: Json<MessageModelNew>,
) -> impl Responder {
    let create = controller_message_create(state.into_inner(), message.into_inner()).await;

    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_message_get_all_by_conversation(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => match controller_message_get_all_by_conversation(state.into_inner(), i).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_message_delete_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    let find = controller_message_delete_by_id(state.into_inner(), id.into_inner()).await;

    match find {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
