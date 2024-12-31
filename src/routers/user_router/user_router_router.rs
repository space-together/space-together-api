use std::sync::Arc;

use actix_web::web::{delete, get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::user_handle::user_handel_handle::{
        handle_create_user, handle_get_all_users, handle_get_user_by_email, handle_get_user_by_id,
        handle_get_user_by_username, handle_user_delete_by_id, handle_user_delete_by_username,
        handle_user_delete_many, handle_user_get_all_by_role, handle_user_update_by_id,
        handle_user_update_by_username, handle_user_update_many,
    },
    AppState,
};

pub fn routers_user(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_create_user))
            .route("", get().to(handle_get_all_users))
            .route("/email/{email}", get().to(handle_get_user_by_email))
            .route(
                "/username/{username}",
                get().to(handle_get_user_by_username),
            )
            .route(
                "/username/{username}",
                delete().to(handle_user_delete_by_username),
            )
            .route(
                "/username/{username}",
                put().to(handle_user_update_by_username),
            )
            .route("/delete-many", post().to(handle_user_delete_many))
            .route("/update-many", post().to(handle_user_update_many))
            .route("/rl/{role}", get().to(handle_user_get_all_by_role))
            .route("/{id}", get().to(handle_get_user_by_id))
            .route("/{id}", delete().to(handle_user_delete_by_id))
            .route("/{id}", put().to(handle_user_update_by_id)),
    )
}
