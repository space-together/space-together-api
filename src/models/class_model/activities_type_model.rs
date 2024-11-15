use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitiesTypeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ty: String,
    pub co: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitiesTypeModelNew {
    pub ty: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitiesTypeModelGet {
    pub id: String,
    pub ty: String,
    pub co: String,
}

impl ActivitiesTypeModel {
    pub fn new(ty: ActivitiesTypeModelNew) -> Self {
        ActivitiesTypeModel {
            id: None,
            ty: ty.ty,
            co: DateTime::now(),
        }
    }

    pub fn format(ty: ActivitiesTypeModel) -> ActivitiesTypeModelGet {
        ActivitiesTypeModelGet {
            id: ty.id.map_or("".to_string(), |id| id.to_string()),
            ty: ty.ty,
            co: ty.co.try_to_rfc3339_string().unwrap_or("".to_string()),
        }
    }
}
