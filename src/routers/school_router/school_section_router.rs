use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::school_handle::school_section_handle::{
        create_school_section_handle, delete_school_section_by_id_handle,
        get_all_school_section_handle, get_school_section_by_id_handle,
        update_school_section_handle,
    },
    AppState,
};

pub fn routers_school_section(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("section")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_school_section_handle))
            .route("", get().to(get_all_school_section_handle))
            .route("/{id}", get().to(get_school_section_by_id_handle))
            .route("/{id}", put().to(update_school_section_handle))
            .route("{id}", delete().to(delete_school_section_by_id_handle)),
    )
}
