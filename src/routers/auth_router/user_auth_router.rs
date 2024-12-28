use std::sync::Arc;

use actix_web::web::{post, scope, Data, ServiceConfig};

use crate::{
    handlers::auth_handle::login_handle::{user_login_handle, user_register_handle},
    AppState,
};

pub fn routers_user_auth_router(
    cfg: &mut ServiceConfig,
    state: Arc<AppState>,
) -> &mut ServiceConfig {
    cfg.service(
        scope("/user")
            .app_data(Data::new(state.clone()))
            .route("/login", post().to(user_login_handle))
            .route("/register", post().to(user_register_handle)),
    )
}
