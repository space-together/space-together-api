use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub sector_id: Option<ObjectId>,
    pub create_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeModelGet {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sector_id: Option<String>,
    pub create_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeModelNew {
    pub name: String,
    pub sector_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeModelPut {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sector_id: Option<String>,
}

impl TradeModel {
    pub fn new(section: TradeModelNew) -> Self {
        TradeModel {
            id: None,
            name: section.name,
            sector_id: section.sector_id.map(|id| ObjectId::from_str(&id).unwrap()),
            description: section.description,
            create_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(section: Self) -> TradeModelGet {
        TradeModelGet {
            id: section.id.map_or("".to_string(), |id| id.to_string()),
            name: section.name,
            description: section.description,
            sector_id: section.sector_id.map(|id| id.to_string()),
            create_on: section
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: section
                .updated_on
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(section: TradeModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", section.name.map(bson::Bson::String));
        insert_if_some("description", section.description.map(bson::Bson::String));

        if is_updated {
            set_doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}
