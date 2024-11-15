use std::sync::Arc;

use actix_web::web::{post, scope, Data, ServiceConfig};

use crate::{handlers::conversation_handle::message_handle::handle_message_create, AppState};

pub fn routers_message(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("conversation")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_message_create)),
    )
}
