use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use std::sync::Arc;

use super::user_router::user_role_router::routers_user_role;
use crate::AppState;

pub fn all_routers(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(
        web::scope("/api/v0.0.1")
            // Use .route() for manual_hello
            .route("/", web::get().to(manual_hello)) // This binds the GET request for "/"
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/user").configure(|user_cfg| {
                routers_user_role(user_cfg, state.clone());
            })),
    );
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
