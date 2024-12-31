use serde::{Deserialize, Serialize};

use crate::models::auth::login_model::UserLoginClaimsModel;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserClaimsModel {
    pub exp: usize,
    pub iat: usize,
    pub user: UserLoginClaimsModel,
}
