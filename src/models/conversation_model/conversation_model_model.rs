use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub mms: Vec<ObjectId>, // Members
    pub gr: Option<bool>,   // Group
    pub co: DateTime,
    pub uo: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationModelNew {
    pub mms: Vec<String>,
    pub gr: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationModelGet {
    pub id: String,
    pub mms: Vec<String>,
    pub gr: Option<bool>,
    pub co: String,
    pub uo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationModelPut {
    pub mms: Option<Vec<String>>,
    pub gr: Option<bool>,
}

impl ConversationModel {
    pub fn new(conversation: ConversationModelNew) -> Self {
        let mms: Vec<ObjectId> = conversation
            .mms
            .iter()
            .filter_map(|s| ObjectId::parse_str(s).ok())
            .collect();

        ConversationModel {
            id: None,
            mms,
            gr: conversation.gr,
            co: DateTime::now(),
            uo: None,
        }
    }
    pub fn format(conversation: ConversationModel) -> ConversationModelGet {
        ConversationModelGet {
            id: conversation.id.map_or("".to_string(), |id| id.to_string()),
            mms: conversation.mms.iter().map(|id| id.to_string()).collect(),
            gr: conversation.gr,
            co: conversation
                .co
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            uo: Some(conversation.uo.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(conversation: ConversationModelPut) -> Document {
        let mut doc = Document::new();

        let mut some_data = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        some_data(
            "mms",
            conversation.mms.map(|ids| {
                bson::Bson::Array(
                    ids.into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        some_data("gr", conversation.gr.map(bson::Bson::Boolean));

        doc
    }
}
