use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::conversation_controller::conversation_controller_controller::{
        controller_conversation_by_id, controller_conversation_create,
    },
    models::{
        conversation_model::conversation_model_model::ConversationModelNew,
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_conversation_create(
    state: Data<AppState>,
    conversation: Json<ConversationModelNew>,
) -> impl Responder {
    let create =
        controller_conversation_create(state.into_inner(), conversation.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_conversation_get_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    let get = controller_conversation_by_id(state.into_inner(), id.into_inner()).await;
    match get {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
