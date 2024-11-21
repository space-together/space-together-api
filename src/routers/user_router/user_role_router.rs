use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::user_handle::user_role_handle::{
        handle_create_user_role, handle_get_all_user_roles, handle_get_user_role,
        handle_user_role_delete, handle_user_role_get_by_name, handle_user_role_update,
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
            .route("/{id}", put().to(handle_user_role_update))
            .route("/{id}", delete().to(handle_user_role_delete))
            .route("/search", get().to(handle_user_role_get_by_name)) //http://127.0.0.1:2052/api/v0.0.1/user/role/search?name=Student
            .route("/{id}", get().to(handle_get_user_role)),
    )
}
