use std::sync::Arc;

use actix_web::web::{delete, get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::activities_type_handler::{
        handle_activity_type_create, handle_activity_type_delete_by_id,
        handle_activity_type_get_all, handle_activity_type_get_by_id,
        handle_activity_type_update_by_id,
    },
    AppState,
};

pub fn routers_activities_type(
    cfg: &mut ServiceConfig,
    state: Arc<AppState>,
) -> &mut ServiceConfig {
    cfg.service(
        scope("/role")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_activity_type_create))
            .route("", get().to(handle_activity_type_get_all))
            .route("/{id}", get().to(handle_activity_type_get_by_id))
            .route("/{id}", put().to(handle_activity_type_update_by_id))
            .route("/{id}", delete().to(handle_activity_type_delete_by_id)),
    )
}
