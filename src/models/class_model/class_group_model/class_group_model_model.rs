use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::error::class_error::class_group_err::{ClassGroupErr, ClassGroupResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nm: String,
    pub cl_id: ObjectId,
    pub co: DateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassGroupModelGet {
    pub id: String,
    pub nm: String,
    pub cl_id: String,
    pub co: String,
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
                co: DateTime::now(),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn format(group: ClassGroupModel) -> ClassGroupModelGet {
        ClassGroupModelGet {
            id: group.id.map_or("".to_string(), |id| id.to_string()),
            cl_id: group.cl_id.to_string(),
            nm: group.nm,
            co: group
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }
}
