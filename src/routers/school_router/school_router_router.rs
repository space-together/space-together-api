use actix_web::web::{self, get, post};
use std::sync::Arc;

use crate::{
    handlers::school_handle::school_handle_handle::{
        create_school_handle, get_school_by_id_handle,
    },
    AppState,
};

pub fn routers_school(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_school_handle))
            .route("/{id}", get().to(get_school_by_id_handle)),
    )
}
