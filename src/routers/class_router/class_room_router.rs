use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::class_handle::class_room_handler::{
        create_class_room_handle, delete_class_room_by_id_handle, get_all_class_room_handle,
        get_class_room_by_id_handle, get_class_room_by_trade_handle, get_class_room_by_trade_type,
        update_class_room_by_id_handle,
    },
    AppState,
};

pub fn routers_class_room(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_class_room_handle))
            .route("", get().to(get_all_class_room_handle))
            .route("type/{id}", get().to(get_class_room_by_trade_type))
            .route("trade/{id}", get().to(get_class_room_by_trade_handle))
            .route("/{id}", get().to(get_class_room_by_id_handle))
            .route("/{id}", delete().to(delete_class_room_by_id_handle))
            .route("{id}", put().to(update_class_room_by_id_handle)),
    )
}
