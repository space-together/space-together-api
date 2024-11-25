use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::error::class_error::class_error_error::{ClassError, ClassResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nm: String,                  // name
    pub cltea: ObjectId,             // teacher id
    pub st: Option<Vec<ObjectId>>,   // Student
    pub teas: Option<Vec<ObjectId>>, //teachers
    pub co: DateTime,                // create on
    pub uo: Option<DateTime>,        // update on
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelNew {
    pub nm: String,
    pub cltea: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelPut {
    pub nm: Option<String>,
    pub cltea: Option<String>,     // teacher id
    pub st: Option<Vec<String>>,   // student id
    pub teas: Option<Vec<String>>, // teachers id
}

impl ClassModel {
    pub fn new(class: ClassModelNew) -> ClassResult<ClassModel> {
        let teacher_id = ObjectId::from_str(&class.cltea).map_err(|_| ClassError::InvalidId);
        match teacher_id {
            Ok(id) => Ok(ClassModel {
                id: None,
                nm: class.nm,
                cltea: id,
                teas: Some(Vec::new()),
                st: Some(Vec::new()),
                co: DateTime::now(),
                uo: None,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn put(class: ClassModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        // Handle updates for `nm` (name) and `cltea` (class teacher)
        insert_if_some("nm", class.nm.map(bson::Bson::String));
        insert_if_some(
            "cltea",
            class
                .cltea
                .map(|cltea| bson::Bson::ObjectId(ObjectId::from_str(&cltea).unwrap())),
        );

        insert_if_some(
            "st",
            class.st.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        insert_if_some(
            "teas",
            class.teas.map(|teachers| {
                bson::Bson::Array(
                    teachers
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        doc
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelGet {
    pub id: String,
    pub nm: String,
    pub cltea: String,
    pub st: Option<Vec<String>>,
    pub teas: Option<Vec<String>>,
    pub co: String,
    pub uo: Option<String>,
}

impl ClassModelGet {
    pub fn format(class: ClassModel) -> ClassModelGet {
        ClassModelGet {
            id: class.id.map_or("".to_string(), |id| id.to_string()),
            nm: class.nm,
            cltea: class.cltea.to_string(),
            st: class
                .st
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            teas: class
                .teas
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            co: class
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            uo: Some(class.uo.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }
}
