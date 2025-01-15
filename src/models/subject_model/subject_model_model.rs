use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub books: Option<Vec<ObjectId>>,
    pub subject_type_id: Option<ObjectId>,
    pub room_id: Option<ObjectId>,
    pub description: Option<String>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelGet {
    pub id: String,
    pub name: String,
    pub books: Option<Vec<String>>,
    pub subject_type_id: Option<String>,
    pub room_id: Option<String>,
    pub description: Option<String>,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelNew {
    pub name: String,
    pub books: Option<Vec<String>>,
    pub subject_type_id: Option<String>,
    pub room_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelPut {
    pub name: Option<String>,
    pub description: Option<String>,
    pub books: Option<Vec<String>>,
    pub subject_type_id: Option<String>,
    pub room_id: Option<String>,
}

impl SubjectModel {
    pub fn new(subject: SubjectModelNew) -> Self {
        SubjectModel {
            id: None,
            name: subject.name,
            description: subject.description,
            books: subject.books.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
            room_id: subject.room_id.map(|r| ObjectId::from_str(&r).unwrap()),
            subject_type_id: subject
                .subject_type_id
                .map(|r| ObjectId::from_str(&r).unwrap()),
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(subject: Self) -> SubjectModelGet {
        SubjectModelGet {
            id: subject.id.map_or("".to_string(), |id| id.to_string()),
            name: subject.name,
            description: subject.description,
            room_id: subject.room_id.map(|id| id.to_string()),
            subject_type_id: subject.subject_type_id.map(|id| id.to_string()),
            books: subject
                .books
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            created_on: subject
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(subject.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(subject: SubjectModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };
        insert_if_some(
            "subject_type_id",
            subject
                .subject_type_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "room_id",
            subject
                .room_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some("name", subject.name.map(bson::Bson::String));
        insert_if_some("description", subject.description.map(bson::Bson::String));
        insert_if_some(
            "books",
            subject.books.map(|teachers| {
                bson::Bson::Array(
                    teachers
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        if is_update {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
