use crate::models::school_token_model::SchoolToken;
use actix_web::{HttpMessage, HttpRequest};

pub fn get_school_id_from_request(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<SchoolToken>()
        .map(|token| token.id.clone())
}
