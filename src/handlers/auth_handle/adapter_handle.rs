use std::collections::HashMap;

use actix_web::{
    web::{Data, Json, Path, Query},
    HttpResponse, Responder,
};
use mongodb::bson::doc;
use serde_json::json;

use crate::{
    models::{
        auth::adapter_model::{AccountModel, AccountModelNew, SessionModel, SessionModelNew},
        request_error_model::ReqErrModel,
    },
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

// async fn update_session(state: Data<AppState>, user_id : Path<String>, session : Json<SessionModelNew>) -> impl Responder {
//     let update = state.db.session.collection.find_one_and_update(doc! {"user_id"}, update)
// }

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

pub async fn unlink_account(
    state: Data<AppState>,
    query: Query<HashMap<String, String>>,
) -> impl Responder {
    if let (Some(provider), Some(provider_account_id)) =
        (query.get("provider"), query.get("providerAccountId"))
    {
        let delete = state
            .db
            .account
            .collection
            .find_one_and_delete(doc! {provider: provider_account_id})
            .await;

        match delete {
            Ok(Some(e)) => HttpResponse::Ok().json(e),
            Ok(None) => HttpResponse::NotFound().json(json!({ "error": "account not found" })),
            Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        }
    } else {
        HttpResponse::BadRequest().json(ReqErrModel {
            message: "Can not delete account".to_string(),
        })
    }
}
