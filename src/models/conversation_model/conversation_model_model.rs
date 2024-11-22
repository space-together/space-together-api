use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub mms: Vec<ObjectId>, // Members
    pub gr: Option<bool>,   // Group
    pub co: DateTime,
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
        }
    }
}
