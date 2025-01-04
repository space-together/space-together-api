use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use sha256::digest;

use crate::{
    controllers::user_controller::user_controller_controller::{
        controller_create_user, controller_user_get_user_by_email,
    },
    libs::{functions::characters_fn::is_valid_email, utils::jwt::jwt_login::user_encode_jwt},
    models::{
        auth::login_model::{UserLoginClaimsModel, UserLoginModule},
        jwt_model::token_model::TokenModel,
        request_error_model::ReqErrModel,
        user_model::user_model_model::UserModelNew,
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

    let get_user = controller_user_get_user_by_email(state.into_inner(), user.email.clone()).await;

    if let Err(e) = get_user {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        });
    }
    let get_user = get_user.unwrap();

    if get_user.password.clone().unwrap() != digest(user.password.clone()) {
        return HttpResponse::Unauthorized().json(ReqErrModel {
            message: "Invalid credentials".to_string(),
        });
    }

    let user_claim = UserLoginClaimsModel {
        id: get_user.id.to_string(),
        name: get_user.name.clone(),
        email: get_user.email.clone(),
        role: get_user.role.to_string(),
    };

    let token = user_encode_jwt(user_claim).unwrap();

    HttpResponse::Ok().json(TokenModel {
        token,
        user: Some(get_user),
    })
}

pub async fn user_register_handle(
    state: Data<AppState>,
    user: Json<UserModelNew>,
) -> impl Responder {
    if let Err(e) = is_valid_email(&user.email.clone()) {
        return HttpResponse::BadRequest().json(ReqErrModel { message: e });
    }

    let create = controller_create_user(user.into_inner(), state.into_inner()).await;
    match create {
        Ok(res) => {
            let user_claim = UserLoginClaimsModel {
                id: res.id.clone(),
                name: res.name.clone(),
                email: res.email.clone(),
                role: res.role.clone(),
            };

            let token = user_encode_jwt(user_claim).unwrap();

            HttpResponse::Ok().json(TokenModel {
                token,
                user: Some(res),
            })
        }
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
