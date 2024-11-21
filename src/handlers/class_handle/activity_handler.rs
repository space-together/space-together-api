use std::str::FromStr;

use actix_web::{
    web::{Data, Path},
    HttpResponse, Responder,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    controllers::class_controller::activity_controller::controller_activity_get_by_class,
    models::request_error_model::ReqErrModel, AppState,
};

pub async fn handle_activity_get_by_class(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match ObjectId::from_str(&id) {
        Ok(obj) => {
            let get_all = controller_activity_get_by_class(state.into_inner(), obj).await;
            match get_all {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                    message: err.to_string(),
                }),
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ReqErrModel::id(id.into_inner())),
    }
}
