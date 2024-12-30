use actix_web::web::{self, get};
use std::sync::Arc;

use crate::{
    handlers::school_handle::{handle_school_get, handle_school_get_by_id},
    AppState,
};

pub fn routers_school(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", get().to(handle_school_get))
            .route("/{id}", get().to(handle_school_get_by_id)),
    )
}
