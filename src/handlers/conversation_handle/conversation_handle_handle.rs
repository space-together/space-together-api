use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;

use crate::{
    controllers::conversation_controller::conversation_controller_controller::{
        controller_conversation_by_id, controller_conversation_by_member,
        controller_conversation_create, controller_conversation_update_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        conversation_model::conversation_model_model::{
            ConversationModelNew, ConversationModelPut,
        },
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
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => match controller_conversation_by_id(state.into_inner(), i).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_conversation_update_by_id(
    state: Data<AppState>,
    id: Path<String>,
    data: Json<ConversationModelPut>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => {
            match controller_conversation_update_by_id(
                state.into_inner(),
                i,
                Some(data.into_inner()),
                None,
                None,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: err.to_string(),
                }),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct MemberModel {
    pub members: Vec<String>,
}

pub async fn handle_conversation_add_member(
    state: Data<AppState>,
    id: Path<String>,
    data: Json<MemberModel>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => {
            match controller_conversation_update_by_id(
                state.into_inner(),
                i,
                None,
                Some(data.into_inner().members),
                None,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: err.to_string(),
                }),
            }
        }
    }
}

pub async fn handle_conversation_remover_member(
    state: Data<AppState>,
    id: Path<String>,
    data: Json<MemberModel>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(i) => {
            match controller_conversation_update_by_id(
                state.into_inner(),
                i,
                None,
                None,
                Some(data.into_inner().members),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: err.to_string(),
                }),
            }
        }
    }
}

pub async fn handle_conversation_get_by_member(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::Ok().json(err),
        Ok(i) => match controller_conversation_by_member(state.into_inner(), i).await {
            Err(err) => HttpResponse::Ok().json(ReqErrModel {
                message: err.to_string(),
            }),
            Ok(data) => HttpResponse::Ok().json(data),
        },
    }
}
