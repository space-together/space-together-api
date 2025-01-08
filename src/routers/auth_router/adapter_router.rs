use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::auth_handle::adapter_handle::{create_session, get_session},
    AppState,
};

pub fn routers_adapter(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("")
            .app_data(Data::new(state.clone()))
            .route("/", post().to(create_session))
            .route("/{token}", get().to(get_session)),
    )
}
