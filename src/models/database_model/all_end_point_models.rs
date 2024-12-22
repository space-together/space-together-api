use serde::Serialize;

// Struct to hold information about each endpoint
#[derive(Debug, Clone, Serialize)]
pub struct EndpointMolder {
    pub method: String,
    pub path: String,
}

// Struct to hold endpoints grouped by categories
#[derive(Debug, Clone, Serialize)]
pub struct EndpointCategoryModel {
    pub name: String,
    pub endpoints: Vec<EndpointMolder>,
}
