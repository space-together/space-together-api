use std::sync::Arc;

use actix_web::web::{self, get, post};

use crate::{
    handlers::user_handle::user_role_handle::{
        handle_create_user_role, handle_get_all_user_roles, handle_get_user_role,
    },
    AppState,
};

pub fn routers_user_role(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("/role")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(handle_create_user_role))
            .route("", get().to(handle_get_all_user_roles))
            .route("/{id}", get().to(handle_get_user_role)),
    )
}
