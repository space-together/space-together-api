use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::education_handle::education_handle_handle::{
        create_education_handle, delete_education_by_id_handle, get_all_education_handle,
        get_education_by_id_handle, update_education_by_id_handle,
    },
    AppState,
};

pub fn routers_education(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_education_handle))
            .route("", get().to(get_all_education_handle))
            .route("/{id}", get().to(get_education_by_id_handle))
            .route("/{id}", delete().to(delete_education_by_id_handle))
            .route("{id}", put().to(update_education_by_id_handle)),
    )
}
