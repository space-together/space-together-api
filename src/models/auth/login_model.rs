use serde::{Deserialize, Serialize};

use crate::models::user_model::user_model_model::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginModule {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLoginClaimsModel {
    pub email: String,
    pub name: String,
    pub role: Option<UserRole>,
    pub id: String,
}
