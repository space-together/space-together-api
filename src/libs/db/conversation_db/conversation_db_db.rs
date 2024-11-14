use std::str::FromStr;

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
}
