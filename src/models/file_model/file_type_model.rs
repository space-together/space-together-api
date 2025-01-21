use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTypeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTypeModelGet {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTypeModelNew {
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTypeModelPut {
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<String>>,
}

impl FileTypeModel {
    pub fn new(fille_type: FileTypeModelNew) -> Self {
        FileTypeModel {
            id: None,
            name: fille_type.name,
            username: fille_type.username,
            description: fille_type.description,
            roles: fille_type.roles,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(fille_type: Self) -> FileTypeModelGet {
        FileTypeModelGet {
            id: fille_type.id.map_or("".to_string(), |id| id.to_string()),
            name: fille_type.name,
            username: fille_type.username,
            description: fille_type.description,
            roles: fille_type.roles,
            created_on: fille_type
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(fille_type.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(fille_type: FileTypeModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some("name", fille_type.name.map(bson::Bson::String));
        insert_if_some("username", fille_type.username.map(bson::Bson::String));
        insert_if_some(
            "description",
            fille_type.description.map(bson::Bson::String),
        );
        if let Some(roles) = fille_type.roles {
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
