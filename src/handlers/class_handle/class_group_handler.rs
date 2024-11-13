use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::{
    controllers::class_controller::class_group_controller::controller_class_group_create,
    models::{
        class_model::class_group_model::class_group_model_model::ClassGroupModelNew,
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_create_class_groups(
    state: Data<AppState>,
    group: Json<ClassGroupModelNew>,
) -> impl Responder {
    let create = controller_class_group_create(state.into_inner(), group.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
