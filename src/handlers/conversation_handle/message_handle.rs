use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::{
    controllers::conversation_controller::message_controller::controller_message_create,
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
