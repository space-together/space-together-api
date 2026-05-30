use crate::config::postgres_manager::PgManager;
use crate::services::event_bus::EventBus;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pg: PgManager,
    pub event_bus: Arc<EventBus>,
}

impl AppState {
    pub fn new(pg: PgManager) -> Self {
        Self {
            pg,
            event_bus: Arc::new(EventBus::new()),
        }
    }
}
