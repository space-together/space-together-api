use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use sha256::digest;

use crate::{
    libs::functions::characters_fn::is_valid_email,
    models::{auth::login_model::UserLoginModule, request_error_model::ReqErrModel},
    AppState,
};

pub async fn user_login_handle(
    state: Data<AppState>,
    user: Json<UserLoginModule>,
) -> impl Responder {
    if let Err(e) = is_valid_email(&user.email.clone()) {
        return HttpResponse::BadRequest().json(ReqErrModel { message: e });
    }

    let get_user = state.db.user.get_user_by_email(user.email.clone()).await;

    if let Err(e) = get_user {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        });
    }
    if get_user.unwrap().pw.unwrap() != digest(user.password.clone()) {
        return HttpResponse::Unauthorized().json(ReqErrModel {
            message: "Invalid credentials".to_string(),
        });
    }

    HttpResponse::Ok().json("{'token': 'hello'}".to_string())
}
