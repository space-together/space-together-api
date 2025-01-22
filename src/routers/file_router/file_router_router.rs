use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::file_handle::file_handle_handle::{
        create_file_handle, delete_file_by_id_handle, get_all_file_handle, get_file_by_id_handle,
        update_file_by_id_handle,
    },
    AppState,
};

pub fn routers_file(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_file_handle))
            .route("", get().to(get_all_file_handle))
            .route("/{id}", get().to(get_file_by_id_handle))
            .route("/{id}", delete().to(delete_file_by_id_handle))
            .route("{id}", put().to(update_file_by_id_handle)),
    )
}
