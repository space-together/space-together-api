use std::sync::Arc;

use actix_web::web::{get, scope, Data, ServiceConfig};

use crate::{handlers::database_handle::database_status_handle::handle_database_status, AppState};

pub fn routers_database(cfg: &mut ServiceConfig, state: Arc<AppState>) -> &mut ServiceConfig {
    cfg.service(scope("/database"))
        .app_data(Data::new(state.clone()))
        .route("", get().to(handle_database_status))
}
