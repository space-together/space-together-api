use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{
    models::{auth::adapter_model::SessionModel, request_error_model::ReqErrModel},
    AppState,
};

pub async fn create_session(state: Data<AppState>, session: Json<SessionModel>) -> impl Responder {
    let create = state
        .db
        .session
        .create(session.into_inner(), Some("collection".to_string()))
        .await;

    match create {
        Err(e) => HttpResponse::InternalServerError().json(json!({"error" : e.to_string()})),
        Ok(_) => HttpResponse::Created().json(json!({"status" : "Session created"})),
    }
}
