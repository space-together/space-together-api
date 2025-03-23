use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::{
    libs::auth::user_register::user_register,
    models::{request_error_model::ReqErrModel, user_model::user_model_model::UserModelNew},
    AppState,
};

pub async fn user_register_router(
    data: Json<UserModelNew>,
    state: Data<AppState>,
) -> impl Responder {
    match user_register(state.into_inner(), data.into_inner()).await {
        Err(e) => HttpResponse::Created().json(ReqErrModel { message: e }),
        Ok(d) => HttpResponse::Created().json(d),
    }
}
