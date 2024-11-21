use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::activity_handler::{
        handle_activity_create, handle_activity_get_by_class, handle_activity_get_by_group,
        handle_activity_get_by_id,
    },
    AppState,
};

pub fn routers_activity(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/activity")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_activity_create))
            .route("/{id}", get().to(handle_activity_get_by_id))
            .route("/class/{id}", get().to(handle_activity_get_by_class))
            .route("/group/{id}", get().to(handle_activity_get_by_group)),
    )
}
