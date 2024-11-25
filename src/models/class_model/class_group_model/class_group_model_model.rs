use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::error::class_error::class_group_err::{ClassGroupErr, ClassGroupResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nm: String,                // name
    pub cl_id: ObjectId,           //class id
    pub st: Option<Vec<ObjectId>>, // students
    pub co: DateTime,              //create on
    pub uo: Option<DateTime>,      // update on
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelGet {
    pub id: String,
    pub nm: String,
    pub cl_id: String,
    pub st: Option<Vec<String>>,
    pub co: String,
    pub uo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelPut {
    pub nm: Option<String>,
    pub cl_id: Option<String>,
    pub st: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelNew {
    pub nm: String,
    pub cl_id: String,
}

impl ClassGroupModel {
    pub fn new(group: ClassGroupModelNew) -> ClassGroupResult<ClassGroupModel> {
        let class_id = ObjectId::from_str(&group.cl_id).map_err(|_| ClassGroupErr::InvalidId);
        match class_id {
            Ok(res) => Ok(ClassGroupModel {
                id: None,
                nm: group.nm,
                cl_id: res,
                st: Some(Vec::new()),
                co: DateTime::now(),
                uo: None,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn format(group: ClassGroupModel) -> ClassGroupModelGet {
        ClassGroupModelGet {
            id: group.id.map_or("".to_string(), |id| id.to_string()),
            cl_id: group.cl_id.to_string(),
            nm: group.nm,
            st: group
                .st
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            co: group
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            uo: Some(group.uo.map_or("".to_string(), |date| {
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
            "st",
            group.st.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        insert_if_some(
            "cl_id",
            group
                .cl_id
                .map(|class| bson::Bson::ObjectId(ObjectId::from_str(&class).unwrap())),
        );

        insert_if_some("nm", group.nm.map(bson::Bson::String));

        doc
    }
}
