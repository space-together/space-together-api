use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};

use crate::{
    controllers::school_controller::school_controller_controller::{
        create_school, get_school_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{request_error_model::ReqErrModel, school_model::school_model_model::SchoolModelNew},
    AppState,
};

pub async fn create_school_handle(
    state: Data<AppState>,
    school: Json<SchoolModelNew>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(token) = req.headers().get("Authorization") {
        match create_school(&state, &school, token.to_str().unwrap()).await {
            Ok(res) => HttpResponse::Created().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        }
    } else {
        HttpResponse::BadRequest().json("Token not found")
    }
}

pub async fn get_school_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match get_school_by_id(&state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}
