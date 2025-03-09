use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::subject_handle::subject_handle_handle::{
        create_subject_handle, delete_subject_by_id_handle, get_all_subject_handle,
        get_subject_by_code_handle, get_subject_by_id_handle, get_subjects_by_class_room_handle,
        update_subject_by_id_handle,
    },
    services::subject_services::create_subject_servicer,
    AppState,
};

// version 0.0.l
pub fn routers_subject(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("/code/{code}", get().to(get_subject_by_code_handle)) // TODO: TO TEST IS GET SUBJECT BY CODE IS IT WORK
            .route(
                "/class-room/{id}",
                get().to(get_subjects_by_class_room_handle),
            ) // TODO: TO TEST IS GET SUBJECT BY CLASS ROOM IS IT WORK
            .route("", post().to(create_subject_handle))
            .route("", get().to(get_all_subject_handle))
            .route("/{id}", get().to(get_subject_by_id_handle))
            .route("/{id}", delete().to(delete_subject_by_id_handle))
            .route("{id}", put().to(update_subject_by_id_handle)),
    )
}

// version 0.0.2
pub fn subject_routers(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_subject_servicer)),
    )
}
