use std::str::FromStr;

use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::conversation_error::conversation_error_error::{ConversationErr, ConversationResult},
    models::conversation_model::conversation_model_model::{
        ConversationModel, ConversationModelNew,
    },
};

#[derive(Debug)]
pub struct ConversationDb {
    pub conversation: Collection<ConversationModel>,
}

impl ConversationDb {
    pub async fn create_conversation(
        &self,
        conversation: ConversationModelNew,
    ) -> ConversationResult<InsertOneResult> {
        let new = ConversationModel::new(conversation);
        let create = self.conversation.insert_one(new).await.map_err(|err| {
            ConversationErr::CanNotCreateConversation {
                err: err.to_string(),
            }
        });
        match create {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }
    pub async fn get_conversation_by_id(
        &self,
        id: String,
    ) -> ConversationResult<ConversationModel> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| ConversationErr::InvalidId);
        match obj_id {
            Ok(res) => {
                let get = self.conversation.find_one(doc! {"_id" : res}).await;
                match get {
                    Ok(Some(result)) => Ok(result),
                    Ok(None) => Err(ConversationErr::ConversationNotFound),
                    Err(err) => Err(ConversationErr::CanNotFindConversation {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn find_many_by_field(
        &self,
        field: &str,
        value: ObjectId,
    ) -> ConversationResult<Vec<ConversationModel>> {
        let query = if field == "st" {
            doc! { "mms": { "$in": [value] } }
        } else {
            doc! { field: value }
        };

        let mut cursor = self.conversation.find(query).await.map_err(|err| {
            ConversationErr::CanNotGetAllByField {
                err: err.to_string(),
                field: field.to_string(),
            }
        })?;

        let mut conversations = Vec::new();
        while let Some(data) = cursor.next().await {
            match data {
                Ok(doc) => conversations.push(doc),
                Err(err) => {
                    return Err(ConversationErr::CanNotGetAllByField {
                        err: err.to_string(),
                        field: field.to_string(),
                    });
                }
            }
        }

        Ok(conversations)
    }

    pub async fn get_conversation_by_member(
        &self,
        id: ObjectId,
    ) -> ConversationResult<Vec<ConversationModel>> {
        self.find_many_by_field("mms", id).await
    }
}
