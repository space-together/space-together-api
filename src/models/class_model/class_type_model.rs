use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassTypeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassTypeModelGet {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassTypeModelNew {
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassTypeModelPut {
    pub name: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

impl ClassTypeModel {
    pub fn new(class_type: ClassTypeModelNew) -> Self {
        ClassTypeModel {
            id: None,
            name: class_type.name,
            description: class_type.description,
            roles: class_type.roles,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(class_type: Self) -> ClassTypeModelGet {
        ClassTypeModelGet {
            id: class_type.id.map_or("".to_string(), |id| id.to_string()),
            name: class_type.name,
            description: class_type.description,
            roles: class_type.roles,
            created_on: class_type
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(class_type.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(class_type: ClassTypeModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some("name", class_type.name.map(bson::Bson::String));
        insert_if_some(
            "description",
            class_type.description.map(bson::Bson::String),
        );
        if let Some(roles) = class_type.roles {
            let existing_roles = doc.get_array("roles").unwrap_or(&vec![]).clone();
            let mut new_roles = existing_roles
                .iter()
                .map(|r| r.as_str().unwrap().to_string())
                .collect::<Vec<String>>();

            for role in roles {
                if let Some(pos) = new_roles.iter().position(|r| r == &role) {
                    new_roles.remove(pos);
                } else {
                    new_roles.push(role);
                }
            }

            doc.insert(
                "roles",
                bson::Bson::Array(new_roles.into_iter().map(bson::Bson::String).collect()),
            );
            is_update = true;
        }

        if is_update {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
