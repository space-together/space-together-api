use serde::Serialize;

// Struct to hold information about each endpoint
#[derive(Debug, Clone, Serialize)]
pub struct EndpointMolder {
    pub method: String,
    pub path: String,
}
