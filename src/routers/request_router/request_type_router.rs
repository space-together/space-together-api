use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::request_handle::request_type_handle::{
        handle_request_type_create, handle_request_type_get_all,
    },
    AppState,
};

pub fn routers_request_type(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/type")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_request_type_create))
            .route("", get().to(handle_request_type_get_all)),
    )
}
