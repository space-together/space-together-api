use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolSectionModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub create_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolSectionModelGet {
    pub id: String,
    pub name: String,
    pub create_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolSectionModelNew {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolSectionModelPut {
    pub name: Option<String>,
}

impl SchoolSectionModel {
    pub fn new(section: SchoolSectionModelNew) -> Self {
        SchoolSectionModel {
            id: None,
            name: section.name,
            create_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(section: Self) -> SchoolSectionModelGet {
        SchoolSectionModelGet {
            id: section.id.map_or("".to_string(), |id| id.to_string()),
            name: section.name,
            create_on: section
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: section
                .updated_on
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(section: SchoolSectionModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", section.name.map(bson::Bson::String));

        if is_updated {
            set_doc.insert("update_on", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}
