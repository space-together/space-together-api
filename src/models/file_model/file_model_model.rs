use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub src: String,
    pub file_type_id: ObjectId,
    pub description: Option<String>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileModelGet {
    pub id: String,
    pub src: String,
    pub file_type: String,
    pub description: Option<String>,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileModelNew {
    pub src: String,
    pub file_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileModelPut {
    pub src: Option<String>,
    pub file_type: Option<String>,
    pub description: Option<String>,
}

impl FileModel {
    pub fn new(file: FileModelNew) -> Self {
        FileModel {
            id: None,
            src: file.src,
            description: file.description,
            file_type_id: ObjectId::from_str(&file.file_type).unwrap(),
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(file: Self) -> FileModelGet {
        FileModelGet {
            id: file.id.map_or("".to_string(), |id| id.to_string()),
            src: file.src,
            description: file.description,
            file_type: file.file_type_id.to_string(),
            created_on: file
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(file.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(file: FileModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some("src", file.src.map(bson::Bson::String));
        insert_if_some("description", file.description.map(bson::Bson::String));
        insert_if_some(
            "file_type_id",
            file.file_type
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );

        if is_update {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
