use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelGet {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelNew {
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelPut {
    pub name: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

impl EducationModel {
    pub fn new(education: EducationModelNew) -> Self {
        EducationModel {
            id: None,
            name: education.name,
            description: education.description,
            roles: education.roles,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(education: Self) -> EducationModelGet {
        EducationModelGet {
            id: education.id.map_or("".to_string(), |id| id.to_string()),
            name: education.name,
            description: education.description,
            roles: education.roles,
            created_on: education
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(education.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(education: EducationModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some("name", education.name.map(bson::Bson::String));
        insert_if_some("description", education.description.map(bson::Bson::String));

        if is_update {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
