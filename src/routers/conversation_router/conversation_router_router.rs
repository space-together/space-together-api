use std::sync::Arc;

use actix_web::web::{get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::conversation_handle::conversation_handle_handle::{
        handle_conversation_create, handle_conversation_get_by_id,
    },
    AppState,
};

pub fn routers_conversation(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("conversation")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_conversation_create))
            .route("/{id}", get().to(handle_conversation_get_by_id)),
    )
}
