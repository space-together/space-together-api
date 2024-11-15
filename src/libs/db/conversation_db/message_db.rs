use core::sync;
use std::str::FromStr;

use futures::StreamExt;
use mongodb::{
    bson::{de, doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::conversation_error::message_error::{MessageError, MessageResult},
    models::conversation_model::message_model::{MessageModel, MessageModelGet, MessageModelNew},
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

    pub async fn get_messages_by_conversation(
        &self,
        id: String,
    ) -> MessageResult<Vec<MessageModelGet>> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| MessageError::InvalidId);

        match obj_id {
            Ok(i) => {
                let cursor = self
                    .message
                    .find(doc! {"cov" : i})
                    .sort(doc! {"co": -1})
                    .await
                    .map_err(|err| MessageError::CanNotGetAllMessagesForConversation {
                        err: err.to_string(),
                    });
                let mut messages: Vec<MessageModelGet> = Vec::new();

                match cursor {
                    Ok(mut res) => {
                        while let Some(result) = res.next().await {
                            match result {
                                Ok(doc) => messages.push(MessageModel::format(doc)),
                                Err(err) => {
                                    return Err(MessageError::CanNotGetAllMessagesForConversation {
                                        err: err.to_string(),
                                    })
                                }
                            }
                        }
                        Ok(messages)
                    }
                    Err(err) => Err(err),
                }
            }
            Err(_) => Err(MessageError::InvalidId),
        }
    }
    pub async fn delete_message_by_id(&self, id: String) -> MessageResult<MessageModel> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| MessageError::InvalidId);
        match obj_id {
            Ok(i) => {
                let delete = self.message.find_one_and_delete(doc! {"_id" : i}).await;
                match delete {
                    Ok(Some(msg)) => Ok(msg),
                    Ok(None) => Err(MessageError::MessageNotFound),
                    Err(err) => Err(MessageError::CanNotDeleteMessage {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }
}
