use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use sha256::digest;

use crate::{
    libs::{functions::characters_fn::is_valid_email, utils::jwt::jwt_login::user_encode_jwt},
    models::{
        auth::login_model::{UserLoginClaimsModel, UserLoginModule},
        jwt_model::token_model::TokenModel,
        request_error_model::ReqErrModel,
    },
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
    let get_user = get_user.unwrap();

    if get_user.pw.unwrap() != digest(user.password.clone()) {
        return HttpResponse::Unauthorized().json(ReqErrModel {
            message: "Invalid credentials".to_string(),
        });
    }

    let user_claim = UserLoginClaimsModel {
        id: get_user.id.unwrap().to_string(),
        name: get_user.nm,
        email: get_user.em,
        role: Some(get_user.rl.unwrap().to_string()),
    };

    let token = user_encode_jwt(user_claim).unwrap();

    HttpResponse::Ok().json(TokenModel { token })
}
