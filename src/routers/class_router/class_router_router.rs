use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::class_handler_handler::{
        handle_create_class, handle_get_class_by_id, handler_get_all_classes,
    },
    AppState,
};

pub fn routers_class(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/class")
            .app_data(Data::new(state.clone()))
            .route("/", post().to(handle_create_class))
            .route("/", get().to(handler_get_all_classes))
            .route("/{id}", get().to(handle_get_class_by_id)),
    )
}
