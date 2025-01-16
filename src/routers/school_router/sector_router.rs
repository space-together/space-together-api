use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::school_handle::sector_handle::{
        create_sector_handle, delete_sector_by_id_handle, get_all_sector_handle,
        get_sector_by_id_handle, update_sector_by_id_handle,
    },
    AppState,
};

pub fn routers_sector(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("type")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_sector_handle))
            .route("", get().to(get_all_sector_handle))
            .route("/{id}", get().to(get_sector_by_id_handle))
            .route("/{id}", delete().to(delete_sector_by_id_handle))
            .route("{id}", put().to(update_sector_by_id_handle)),
    )
}
