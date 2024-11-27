use std::sync::Arc;

use actix_web::web::{delete, get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::class_group_handler::{
        handle_class_group_add_students, handle_class_group_delete_by_id,
        handle_class_group_get_all, handle_class_group_remove_students,
        handle_class_group_update_by_id, handle_create_class_groups,
        handle_get_class_group_by_class, handle_get_class_group_by_id,
        handle_get_class_group_by_student,
    },
    AppState,
};

pub fn routers_class_group(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/groups")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_create_class_groups))
            .route("", get().to(handle_class_group_get_all))
            .route("/{id}", get().to(handle_get_class_group_by_id))
            .route("/{id}", put().to(handle_class_group_update_by_id))
            .route("/{id}", delete().to(handle_class_group_delete_by_id))
            .route("/class/{id}", get().to(handle_get_class_group_by_class))
            .route(
                "/student/add/{id}",
                post().to(handle_class_group_add_students),
            )
            .route(
                "/student/remove/{id}",
                post().to(handle_class_group_remove_students),
            )
            .route("/student/{id}", get().to(handle_get_class_group_by_student)),
    )
}
