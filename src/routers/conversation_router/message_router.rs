use std::sync::Arc;

use actix_web::web::{delete, get, post, scope, Data, ServiceConfig};

use crate::{
    handlers::conversation_handle::message_handle::{
        handle_message_create, handle_message_delete_by_id, handle_message_get_all_by_conversation,
    },
    AppState,
};

pub fn routers_message(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/messages")
            .app_data(Data::new(state.clone()))
            .route("", post().to(handle_message_create))
            .route("/{id}", delete().to(handle_message_delete_by_id))
            .route(
                "/cov/{id}",
                get().to(handle_message_get_all_by_conversation),
            ),
    )
}
