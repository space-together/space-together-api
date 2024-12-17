use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestTypeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub role: String,
    pub des: Option<String>,
    pub co: DateTime,
    pub uo: Option<DateTime>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RequestTypeModelNew {
    pub role: String,
    pub des: Option<String>,
}

pub struct RequestTypeModelPut {
    pub role: Option<String>,
    pub des: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RequestTypeModelGet {
    pub id: String,
    pub role: String,
    pub des: Option<String>,
    pub co: String,
    pub uo: Option<String>,
}

impl RequestTypeModel {
    pub fn new(role: RequestTypeModelNew) -> Self {
        RequestTypeModel {
            id: None,
            role: role.role,
            des: role.des,
            co: DateTime::now(),
            uo: None,
        }
    }
    pub fn put(role: RequestTypeModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        insert_if_some("role", role.role.map(bson::Bson::String));
        insert_if_some("des", role.des.map(bson::Bson::String));

        doc
    }

    pub fn format(request: RequestTypeModel) -> RequestTypeModelGet {
        RequestTypeModelGet {
            id: request.id.map_or("".to_string(), |id| id.to_string()),
            role: request.role,
            des: request.des,
            co: request.co.try_to_rfc3339_string().unwrap_or("".to_string()),
            uo: Some(request.uo.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }
}
