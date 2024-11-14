use actix_web::{
    web::{self, scope, ServiceConfig},
    HttpResponse, Responder,
};
use std::sync::Arc;

use super::{
    class_router::{class_group_router::routers_class_group, class_router_router::routers_class},
    conversation_router::conversation_router_router::routers_conversation,
    user_router::{user_role_router::routers_user_role, user_router_router::routers_user},
};
use crate::AppState;

pub fn all_routers(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(
        scope("/api/v0.0.1")
            .route("/", web::get().to(manual_hello))
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/user").configure(|user_cfg| {
                routers_user_role(user_cfg, state.clone());
                routers_user(user_cfg, state.clone());
            }))
            .service(web::scope("/class").configure(|user_cfg| {
                routers_class(user_cfg, state.clone());
                routers_class_group(user_cfg, state.clone());
            }))
            .service(web::scope("/conversation").configure(|user_cfg| {
                routers_conversation(user_cfg, state);
            })),
    );
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there! 🌼 this is space-together api")
}
