use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub disabled: Option<bool>,
    pub roles: Option<Vec<String>>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelGet {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub disabled: Option<bool>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelNew {
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub disabled: Option<bool>,
    pub symbol: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationModelPut {
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub disabled: Option<bool>,
    pub roles: Option<Vec<String>>,
}

impl EducationModel {
    pub fn new(education: EducationModelNew) -> Self {
        EducationModel {
            id: None,
            name: education.name,
            username: education.username,
            symbol: education.symbol,
            description: education.description,
            roles: education.roles,
            disabled: education.disabled,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    pub fn format(education: Self) -> EducationModelGet {
        EducationModelGet {
            id: education.id.map_or("".to_string(), |id| id.to_string()),
            name: education.name,
            username: education.username,
            description: education.description,
            symbol: education.symbol,
            roles: education.roles,
            disabled: education.disabled,
            created_at: education
                .created_at
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_at: Some(education.updated_at.map_or("".to_string(), |date| {
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

        // Insert optional fields
        insert_if_some("name", education.name.map(bson::Bson::String));
        insert_if_some("disabled", education.disabled.map(bson::Bson::Boolean));
        insert_if_some(
            "roles",
            education.roles.map(|roles| {
                bson::Bson::Array(roles.into_iter().map(bson::Bson::String).collect())
            }),
        );
        insert_if_some("username", education.username.map(bson::Bson::String));
        insert_if_some("description", education.description.map(bson::Bson::String));

        insert_if_some("symbol", education.symbol.map(bson::Bson::String));

        // Add the `updated_at` field if any fields were updated
        if is_update {
            doc.insert("updated_at", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
