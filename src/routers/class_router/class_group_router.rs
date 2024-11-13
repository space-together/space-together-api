use std::sync::Arc;

use actix_web::web::{post, scope, Data, ServiceConfig};

use crate::{handlers::class_handle::class_group_handler::handle_create_class_groups, AppState};

pub fn routers_class_group(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/groups")
            .app_data(Data::new(state.clone()))
            .route("/", post().to(handle_create_class_groups)),
    )
}
