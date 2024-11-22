use std::sync::Arc;

use actix_web::web::{get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::user_handle::user_handel_handle::{
        handle_create_user, handle_get_all_users, handle_get_user_by_id,
        handle_user_get_all_by_role, handle_user_update_by_id,
    },
    AppState,
};

pub fn routers_user(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/user")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_create_user))
            .route("", get().to(handle_get_all_users))
            .route("/search", get().to(handle_user_get_all_by_role)) //http://127.0.0.1:2052/api/v0.0.1/user/user/search?name=Student
            .route("/{id}", get().to(handle_get_user_by_id))
            .route("/{id}", put().to(handle_user_update_by_id)),
    )
}
