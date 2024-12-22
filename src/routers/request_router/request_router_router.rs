use std::sync::Arc;

use actix_web::web::{delete, get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::request_handle::request_handle_handle::{
        handle_request_create, handle_request_delete, handle_request_get_all,
        handle_request_get_by_id,
    },
    AppState,
};

pub fn routers_request(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("")
            .app_data(Data::new(state))
            .route("", post().to(handle_request_create))
            .route("", get().to(handle_request_get_all))
            .route("/{id}", get().to(handle_request_get_by_id))
            .route("/{id}", delete().to(handle_request_delete)),
    )
}
