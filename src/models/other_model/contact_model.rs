use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMediaModel {
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub youtube: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactModel {
    pub primary_phone: String,
    pub secondary_phone: Option<String>,
    pub emergency_contact: Option<String>,
    pub primary_email: String, // Primary contact email of the school
    pub secondary_email: Option<String>, // Secondary contact email, if any
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactPersonModel {
    pub name: String,             // Name of the contact person
    pub phone: String,            // Phone number of the contact person
    pub email: String,            // Email of the contact person
    pub position: Option<String>, // Position or role of the contact person
}
