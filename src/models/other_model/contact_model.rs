use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMedia {
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub youtube: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactModel {
    pub phone: Option<String>,
    pub email: Option<String>,
}
