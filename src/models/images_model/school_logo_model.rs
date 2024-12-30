use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub school_id: ObjectId, // school id
    pub src: String,
    pub create_on: DateTime, // create on
    pub update_on: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogoGet {
    pub id: String,
    pub school_id: String, // school id
    pub src: String,
    pub create_on: String, // create on
    pub update_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogoNew {
    pub school_id: String, // school id
    pub src: String,
}

impl SchoolLogo {
    pub fn new(logo: SchoolLogoNew) -> Self {
        SchoolLogo {
            id: None,
            school_id: ObjectId::from_str(&logo.school_id).unwrap(),
            src: logo.src,
            create_on: DateTime::now(),
            update_on: None,
        }
    }
}
