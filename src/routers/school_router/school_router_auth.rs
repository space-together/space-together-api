use actix_web::web::{self, post};
use actix_web_lab::middleware::from_fn;
use std::sync::Arc;

use crate::{
    handlers::school_handle::handle_school_create,
    middleware::user_auth_middleware::check_user_auth_middleware, AppState,
};

pub fn routers_school_auth(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("/auth")
            .app_data(web::Data::new(state.clone()))
            .wrap(from_fn(check_user_auth_middleware))
            .route("", post().to(handle_school_create)),
    )
}
