use std::sync::Arc;

use actix_web::web::{get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::class_handler_handler::{
        handle_class_add_students, handle_class_gets_by_student, handle_class_gets_by_teacher,
        handle_class_remove_students, handle_class_update_by_id, handle_create_class,
        handle_get_class_by_id, handler_get_all_classes,
    },
    AppState,
};

pub fn routers_class(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/class")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_create_class))
            .route("", get().to(handler_get_all_classes))
            .route("/{id}", get().to(handle_get_class_by_id))
            .route("/{id}", put().to(handle_class_update_by_id))
            .route("/student/{id}", get().to(handle_class_gets_by_student))
            .route("/student/add/{id}", post().to(handle_class_add_students))
            .route(
                "/student/remove/{id}",
                post().to(handle_class_remove_students),
            )
            .route("/teacher/{id}", get().to(handle_class_gets_by_teacher)),
    )
}
