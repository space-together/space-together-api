use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserClaimsModel {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
    pub name: String,
    pub role: Option<String>,
    pub id: String,
}
