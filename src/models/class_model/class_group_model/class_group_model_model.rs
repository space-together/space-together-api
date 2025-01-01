use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::error::class_error::class_group_err::{ClassGroupErr, ClassGroupResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,                    // name
    pub class_id: ObjectId,              //class id
    pub students: Option<Vec<ObjectId>>, // students
    pub created_on: DateTime,            //create on
    pub update_on: Option<DateTime>,     // update on
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelGet {
    pub id: String,
    pub name: String,
    pub class_id: String,
    pub students: Option<Vec<String>>,
    pub created_on: String,
    pub update_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelPut {
    pub name: Option<String>,
    pub class_id: Option<String>,
    pub students: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelNew {
    pub name: String,
    pub class_id: String,
}

impl ClassGroupModel {
    pub fn new(group: ClassGroupModelNew) -> ClassGroupResult<ClassGroupModel> {
        let class_id = ObjectId::from_str(&group.class_id).map_err(|_| ClassGroupErr::InvalidId);
        match class_id {
            Ok(res) => Ok(ClassGroupModel {
                id: None,
                name: group.name,
                class_id: res,
                students: Some(Vec::new()),
                created_on: DateTime::now(),
                update_on: None,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn format(group: ClassGroupModel) -> ClassGroupModelGet {
        ClassGroupModelGet {
            id: group.id.map_or("".to_string(), |id| id.to_string()),
            class_id: group.class_id.to_string(),
            name: group.name,
            students: group
                .students
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            created_on: group
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            update_on: Some(group.update_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(group: ClassGroupModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        insert_if_some(
            "students",
            group.students.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        insert_if_some(
            "class_id",
            group
                .class_id
                .map(|class| bson::Bson::ObjectId(ObjectId::from_str(&class).unwrap())),
        );

        insert_if_some("name", group.name.map(bson::Bson::String));

        doc
    }
}
