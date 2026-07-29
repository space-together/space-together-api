use crate::models::api_request_model::RequestQuery;
use actix_web::HttpResponse;

// Validates field/value pair counts; returns error response if mismatched.
// Kept for API handlers that do their own field-based filtering.
pub fn validate_field_value_pairs(query: &RequestQuery) -> Option<HttpResponse> {
    if query.field.len() != query.value.len() {
        return Some(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Number of fields must match number of values"
        })));
    }
    None
}
