use actix_web::{web::Data, HttpResponse, Responder};

use crate::{error::db_error::DbError, models::request_error_model::ReqErrModel, AppState};

pub async fn handle_database_status(state: Data<AppState>) -> impl Responder {
    match state.db.stats.clone() {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::NotFound().json(ReqErrModel {
            message: DbError::DatabaseStatusNotFound.to_string(),
        }),
    }
}
