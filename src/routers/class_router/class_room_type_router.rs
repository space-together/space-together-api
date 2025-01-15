use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::class_handle::class_room_type_handler::{
        create_class_room_type_handle, delete_class_room_type_by_id_handle,
        get_all_class_room_type_handle, get_class_room_type_by_id_handle,
        update_class_room_type_by_id_handle,
    },
    AppState,
};

pub fn routers_class_room_type(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("type")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_class_room_type_handle))
            .route("", get().to(get_all_class_room_type_handle))
            .route("/{id}", get().to(get_class_room_type_by_id_handle))
            .route("/{id}", delete().to(delete_class_room_type_by_id_handle))
            .route("{id}", put().to(update_class_room_type_by_id_handle)),
    )
}
