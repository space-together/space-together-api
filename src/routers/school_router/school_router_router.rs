use actix_web::web::{self, get, post};
use actix_web_lab::middleware::from_fn;
use std::sync::Arc;

use crate::{
    handlers::school_handle::{handle_school_create, handle_school_get, handle_school_get_by_id},
    middleware::user_auth_middleware::check_user_auth_middleware,
    AppState,
};

pub fn routers_school(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .wrap(from_fn(check_user_auth_middleware))
            .route("", post().to(handle_school_create))
            .route("", get().to(handle_school_get))
            .route("/{id}", get().to(handle_school_get_by_id)),
    )
}
