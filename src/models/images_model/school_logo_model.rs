use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogoModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub school_id: ObjectId, // school id
    pub src: String,
    pub create_on: DateTime, // create on
    pub update_on: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogoModelGet {
    pub id: String,
    pub school_id: String, // school id
    pub src: String,
    pub create_on: String, // create on
    pub update_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolLogoModelNew {
    pub school_id: String, // school id
    pub src: String,
}

impl SchoolLogoModel {
    pub fn new(logo: SchoolLogoModelNew) -> Self {
        SchoolLogoModel {
            id: None,
            school_id: ObjectId::from_str(&logo.school_id).unwrap(),
            src: logo.src,
            create_on: DateTime::now(),
            update_on: None,
        }
    }

    pub fn format(logo: Self) -> SchoolLogoModelGet {
        SchoolLogoModelGet {
            id: logo.id.map_or("".to_string(), |id| id.to_string()),
            school_id: logo.school_id.to_string(),
            src: logo.src,
            create_on: logo
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            update_on: Some(
                logo.create_on
                    .try_to_rfc3339_string()
                    .unwrap_or("".to_string()),
            ),
        }
    }
}
