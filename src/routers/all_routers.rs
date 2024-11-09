use actix_web::{
    web::{self, scope, ServiceConfig},
    HttpResponse, Responder,
};
use std::sync::Arc;

use super::user_router::{user_role_router::routers_user_role, user_router_router::routers_user};
use crate::AppState;

pub fn all_routers(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(
        scope("/api/v0.0.1")
            .route("/", web::get().to(manual_hello))
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/user").configure(|user_cfg| {
                routers_user_role(user_cfg, state.clone());
                routers_user(user_cfg, state);
            })),
    );
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there! 🌼 this is space-together api")
}
