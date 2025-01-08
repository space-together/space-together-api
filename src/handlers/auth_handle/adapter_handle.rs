use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use mongodb::bson::doc;
use serde_json::json;

use crate::{
    models::auth::adapter_model::{AccountModel, AccountModelNew, SessionModel, SessionModelNew},
    AppState,
};

pub async fn create_session(
    state: Data<AppState>,
    session: Json<SessionModelNew>,
) -> impl Responder {
    let create = state
        .db
        .session
        .create(
            SessionModel::new(session.into_inner()),
            Some("collection".to_string()),
        )
        .await;

    match create {
        Err(e) => HttpResponse::InternalServerError().json(json!({"error" : e.to_string()})),
        Ok(_) => HttpResponse::Created().json(json!({"status" : "Session created"})),
    }
}

pub async fn get_session(state: Data<AppState>, session_token: Path<String>) -> impl Responder {
    match state
        .db
        .session
        .collection
        .find_one(doc! {"session_token": &session_token.into_inner()})
        .await
    {
        Ok(Some(session)) => HttpResponse::Ok().json(session),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": "Session not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

pub async fn link_account(state: Data<AppState>, account: Json<AccountModelNew>) -> impl Responder {
    let create = state
        .db
        .account
        .create(
            AccountModel::new(account.into_inner()),
            Some("collection".to_string()),
        )
        .await;

    match create {
        Err(e) => HttpResponse::InternalServerError().json(json!({"error" : e.to_string()})),
        Ok(_) => HttpResponse::Created().json(json!({"status" : "account created"})),
    }
}

// pub async fn
