use serde::{Deserialize, Serialize};

use crate::models::user_model::user_model_model::UserModelGet;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenModel {
    pub token: String,
    pub user: Option<UserModelGet>,
}
