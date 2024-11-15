use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::class_handle::activities_type_handler::{
        handle_activity_type_create, handle_activity_type_get_all, handle_activity_type_get_by_id,
    },
    AppState,
};

pub fn routers_activities_type(
    cfg: &mut ServiceConfig,
    state: Arc<AppState>,
) -> &mut ServiceConfig {
    cfg.service(
        scope("/activities_type")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_activity_type_create))
            .route("", get().to(handle_activity_type_get_all))
            .route("/{id}", get().to(handle_activity_type_get_by_id)),
    )
}
