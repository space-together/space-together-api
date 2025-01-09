use std::sync::Arc;

use actix_web::web::{delete, get, post, put, scope, Data, ServiceConfig};

use crate::{
    handlers::auth_handle::adapter_handle::{
        create_session, delete_session, get_session_and_user, link_account, unlink_account,
        update_session,
    },
    AppState,
};

pub fn routers_adapter(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("")
            .app_data(Data::new(state.clone()))
            .route("/accounts", post().to(link_account))
            .route("/accounts", delete().to(unlink_account))
            .route("/sessions", post().to(create_session))
            .route("/sessions/{token}", get().to(get_session_and_user))
            .route("/sessions/{user_id}", put().to(update_session))
            .route("/sessions/{session_token}", delete().to(delete_session)),
    )
}
