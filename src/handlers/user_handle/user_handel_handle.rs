use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::{
    controllers::user_controller::user_controller_controller::controller_create_user,
    models::{request_error_model::ReqErrModel, user_model::user_model_model::UserModelNew},
    AppState,
};

pub async fn handle_create_user(state: Data<AppState>, user: Json<UserModelNew>) -> impl Responder {
    let create = controller_create_user(user.into_inner(), state.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
