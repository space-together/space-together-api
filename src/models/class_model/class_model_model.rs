use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::error::class_error::class_error_error::{ClassError, ClassResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nm: String,
    pub cltea: ObjectId,
    pub st: Option<Vec<ObjectId>>,
    pub co: DateTime,
    pub up: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelNew {
    pub nm: String,
    pub cltea: String,
}

impl ClassModel {
    pub fn new(class: ClassModelNew) -> ClassResult<ClassModel> {
        let teacher_id = ObjectId::from_str(&class.cltea).map_err(|_| ClassError::InvalidId);
        match teacher_id {
            Ok(id) => Ok(ClassModel {
                id: None,
                nm: class.nm,
                cltea: id,
                st: None,
                co: DateTime::now(),
                up: None,
            }),
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelGet {
    pub id: String,
    pub nm: String,
    pub cltea: String,
    pub st: Option<Vec<String>>,
    pub co: String,
    pub up: Option<String>,
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
            co: class
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            up: Some(class.up.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }
}
