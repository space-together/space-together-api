use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SectorModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub education_id: Option<ObjectId>,
    pub username: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub create_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectorModelGet {
    pub id: String,
    pub name: String,
    pub education_id: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub create_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectorModelNew {
    pub name: String,
    pub education_id: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectorModelPut {
    pub name: Option<String>,
    pub education_id: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

impl SectorModel {
    pub fn new(section: SectorModelNew) -> Self {
        SectorModel {
            id: None,
            education_id: section
                .education_id
                .map(|id| ObjectId::from_str(&id).unwrap()),
            username: section.username,
            name: section.name,
            description: section.description,
            create_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(section: Self) -> SectorModelGet {
        SectorModelGet {
            id: section.id.map_or("".to_string(), |id| id.to_string()),
            name: section.name,
            education_id: section.education_id.map(|id| id.to_string()),
            username: section.username,
            description: section.description,
            create_on: section
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: section
                .updated_on
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(section: SectorModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", section.name.map(bson::Bson::String));
        insert_if_some("username", section.username.map(bson::Bson::String));
        insert_if_some(
            "education_id",
            section
                .education_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some("description", section.description.map(bson::Bson::String));

        if is_updated {
            set_doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}
