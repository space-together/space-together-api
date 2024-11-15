use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::conversation_error::message_error::{MessageError, MessageResult},
    models::conversation_model::message_model::{MessageModel, MessageModelNew},
};

#[derive(Debug)]
pub struct MessageDb {
    pub message: Collection<MessageModel>,
}

impl MessageDb {
    pub async fn create_message(&self, message: MessageModelNew) -> MessageResult<InsertOneResult> {
        let new = MessageModel::new(message);

        let create =
            self.message
                .insert_one(new)
                .await
                .map_err(|err| MessageError::CanNotCreateMessage {
                    err: err.to_string(),
                });

        match create {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn get_message_by_id(&self, id: String) -> MessageResult<MessageModel> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| MessageError::InvalidId);

        match obj_id {
            Ok(id) => {
                let get = self
                    .message
                    .find_one(doc! {"_id" : id})
                    .await
                    .map_err(|err| MessageError::CanNotFindMessage {
                        err: err.to_string(),
                    });
                match get {
                    Ok(Some(msg)) => Ok(msg),
                    Ok(None) => Err(MessageError::MessageNotFound),
                    Err(err) => Err(MessageError::CanNotFindMessage {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }
}
