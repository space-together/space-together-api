use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    config::state::AppState, models::request_error_model::ReqErrModel,
    services::database_status_service::get_postgres_stats, utils::request_context::postgres_pool,
};

#[get("/status")]
pub async fn db_status(state: web::Data<AppState>) -> impl Responder {
    let service = get_postgres_stats(postgres_pool(&state), None).await;

    match service {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}")]
pub async fn school_db_status(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let school_id = path.into_inner();

    match get_postgres_stats(postgres_pool(&state), Some(&school_id)).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/database")
            .service(db_status)
            .service(school_db_status),
    );
}
