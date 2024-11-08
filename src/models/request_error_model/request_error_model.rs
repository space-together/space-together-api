use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ReqErrModel {
    pub message: String,
}
