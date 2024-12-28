use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginModule {
    pub email: String,
    pub password: String,
}
