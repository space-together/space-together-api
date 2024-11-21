use std::sync::Arc;

use actix_web::web::{get, scope, Data, ServiceConfig};

use crate::{handlers::class_handle::activity_handler::handle_activity_get_by_class, AppState};

pub fn routers_activity(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(
        scope("/activity")
            .app_data(Data::new(state.clone()))
            .route("/class/{id}", get().to(handle_activity_get_by_class)),
    )
}
