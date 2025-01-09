use std::collections::HashMap;

use actix_web::{
    web::{Data, Json, Path, Query},
    HttpResponse, Responder,
};
use mongodb::bson::doc;
use serde_json::json;

use crate::{
    controllers::user_controller::user_controller_controller::controller_get_user_by_id,
    libs::functions::{characters_fn::is_date_string, object_id::change_string_into_object_id},
    models::{
        auth::adapter_model::{
            AccountModel, AccountModelNew, SessionModel, SessionModelNew, SessionModelPut,
        },
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn create_session(
    state: Data<AppState>,
    session: Json<SessionModelNew>,
) -> impl Responder {
    if !is_date_string(&session.expires) {
        return HttpResponse::BadRequest()
            .json(json!({"error" : "expires is not date please use real date example: [2025-01-15T12:00:00Z] "}));
    }

    let user_id = match change_string_into_object_id(session.user_id.clone()) {
        Err(e) => return HttpResponse::BadRequest().json(e),
        Ok(i) => i,
    };

    if let Err(e) = controller_get_user_by_id(state.clone().into_inner(), user_id).await {
        return HttpResponse::BadRequest().json(json!({"error" : e.to_string()}));
    }

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
        Ok(id) => match state
            .db
            .session
            .get_one_by_id(id, Some("Sessions".to_string()))
            .await
        {
            Err(e) => HttpResponse::Created().json(json!({"error" : e.to_string()})),
            Ok(r) => HttpResponse::Created().json(SessionModel::format(r)),
        },
    }
}

pub async fn get_session_and_user(
    state: Data<AppState>,
    session_token: Path<String>,
) -> impl Responder {
    let session_result = state
        .db
        .session
        .collection
        .find_one(doc! { "session_token": session_token.into_inner() })
        .await;

    match session_result {
        Ok(Some(session)) => {
            let user_result =
                controller_get_user_by_id(state.clone().into_inner(), session.user_id).await;

            match user_result {
                Ok(r) => HttpResponse::Ok().json(r),
                Err(e) => HttpResponse::BadRequest().json(json!({"error" : e.to_string()})),
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_session(state: Data<AppState>, session_token: Path<String>) -> impl Responder {
    match state
        .db
        .session
        .collection
        .find_one_and_delete(doc! {"session_token" : &session_token.into_inner()})
        .await
    {
        Ok(Some(r)) => HttpResponse::Ok().json(SessionModel::format(r)),
        Ok(None) => HttpResponse::NotFound().body("session not found"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_session(
    state: Data<AppState>,
    user_id: Path<String>,
    session: Json<SessionModelPut>,
) -> impl Responder {
    let update = state
        .db
        .session
        .collection
        .find_one_and_update(
            doc! {"user_id" : user_id.into_inner()},
            SessionModel::put(session.into_inner()),
        )
        .await;

    match update {
        Ok(Some(session)) => HttpResponse::Ok().json(session),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": "Session not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

pub async fn link_account(state: Data<AppState>, account: Json<AccountModelNew>) -> impl Responder {
    let user_id = match change_string_into_object_id(account.clone().user_id.clone()) {
        Err(e) => return HttpResponse::BadRequest().json(e),
        Ok(i) => i,
    };

    if let Err(e) = controller_get_user_by_id(state.clone().into_inner(), user_id).await {
        return HttpResponse::BadRequest().json(json!({"error" : e.to_string()}));
    }

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
        Ok(id) => match state
            .db
            .account
            .get_one_by_id(id, Some("accounts".to_string()))
            .await
        {
            Err(e) => HttpResponse::Created().json(json!({"error" : e.to_string()})),
            Ok(r) => HttpResponse::Created().json(AccountModel::format(r)),
        },
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
