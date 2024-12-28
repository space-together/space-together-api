use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginModule {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLoginClaimsModel {
    pub email: String,
    pub name: String,
    pub role: Option<String>,
    pub id: String,
}
