use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::class_group_handler::{
        handle_class_group_get_all, handle_create_class_groups, handle_get_class_group_by_id,
    },
    AppState,
};

pub fn routers_class_group(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/groups")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_create_class_groups))
            .route("", get().to(handle_class_group_get_all))
            .route("/{id}", get().to(handle_get_class_group_by_id)),
    )
}
