use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitiesTypeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ty: String,           //type
    pub co: DateTime,         // creation on
    pub uo: Option<DateTime>, // update on
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
    pub uo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitiesTypeModelPut {
    pub ty: Option<String>,
}

impl ActivitiesTypeModel {
    pub fn new(ty: ActivitiesTypeModelNew) -> Self {
        ActivitiesTypeModel {
            id: None,
            ty: ty.ty,
            co: DateTime::now(),
            uo: None,
        }
    }

    pub fn format(ty: ActivitiesTypeModel) -> ActivitiesTypeModelGet {
        ActivitiesTypeModelGet {
            id: ty.id.map_or("".to_string(), |id| id.to_string()),
            ty: ty.ty,
            co: ty.co.try_to_rfc3339_string().unwrap_or("".to_string()),
            uo: Some(ty.uo.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(ty: ActivitiesTypeModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        insert_if_some("ty", ty.ty.map(bson::Bson::String));

        doc
    }
}
