use std::sync::Arc;

use actix_web::web::{post, scope, Data, ServiceConfig};

use crate::{handlers::auth_handle::login_handle::user_login_handle, AppState};

pub fn routers_auth_router(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/auth")
            .app_data(Data::new(state.clone()))
            .route("/login", post().to(user_login_handle)),
    )
}
