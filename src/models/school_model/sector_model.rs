use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SectorModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub education_id: Option<ObjectId>,
    pub username: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub symbol_id: Option<ObjectId>,
    pub create_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SectorModelGet {
    pub id: String,
    pub name: String,
    pub education: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub create_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SectorModelNew {
    pub name: String,
    pub education: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SectorModelPut {
    pub name: Option<String>,
    pub education: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub symbol: Option<String>,
}

impl SectorModel {
    pub fn new(sector: SectorModelNew) -> Self {
        SectorModel {
            id: None,
            education_id: sector.education.map(|id| ObjectId::from_str(&id).unwrap()),
            username: sector.username,
            name: sector.name,
            description: sector.description,
            symbol_id: sector.symbol.map(|id| ObjectId::from_str(&id).unwrap()),
            create_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(sector: Self) -> SectorModelGet {
        SectorModelGet {
            id: sector.id.map_or("".to_string(), |id| id.to_string()),
            name: sector.name,
            education: sector.education_id.map(|id| id.to_string()),
            username: sector.username,
            description: sector.description,
            symbol: sector.symbol_id.map(|id| id.to_string()),
            create_on: sector
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: sector
                .updated_on
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(sector: SectorModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", sector.name.map(bson::Bson::String));
        insert_if_some("username", sector.username.map(bson::Bson::String));
        insert_if_some(
            "education_id",
            sector
                .education
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some("description", sector.description.map(bson::Bson::String));
        insert_if_some(
            "symbol_id",
            sector
                .symbol
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        if is_updated {
            set_doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}
