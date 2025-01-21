use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::school_handle::trade_handle::{
        create_trade_handle, delete_trade_by_id_handle, get_all_trade_handle,
        get_trade_by_id_handle, get_trade_by_sector_handle, update_trade_handle,
    },
    AppState,
};

pub fn routers_trade(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("trade")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_trade_handle))
            .route("", get().to(get_all_trade_handle))
            .route("/sector/{id}", get().to(get_trade_by_sector_handle))
            .route("/{id}", get().to(get_trade_by_id_handle))
            .route("/{id}", put().to(update_trade_handle))
            .route("{id}", delete().to(delete_trade_by_id_handle)),
    )
}
