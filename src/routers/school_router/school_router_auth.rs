use actix_web::web::{self, post};
use std::sync::Arc;

use crate::{handlers::school_handle::school_handle_handle::handle_school_create, AppState};

pub fn routers_school_auth(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("/auth")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(handle_school_create)),
    )
}
