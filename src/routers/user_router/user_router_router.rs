use std::sync::Arc;

use actix_web::web::{post, scope, Data, ServiceConfig};

use crate::{handlers::user_handle::user_handel_handle::handle_create_user, AppState};

pub fn routers_user(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/user")
            .app_data(Data::new(state.clone()))
            .route("/", post().to(handle_create_user)),
    )
}
