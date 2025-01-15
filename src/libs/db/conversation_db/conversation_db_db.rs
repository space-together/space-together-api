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
        ConversationModel, ConversationModelNew, ConversationModelPut,
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
        id: ObjectId,
    ) -> ConversationResult<ConversationModel> {
        match self.conversation.find_one(doc! {"_id" : id}).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ConversationErr::ConversationNotFound),
            Err(err) => Err(ConversationErr::CanNotFindConversation {
                err: err.to_string(),
            }),
        }
    }

    pub async fn delete_conversation_by_id(
        &self,
        id: ObjectId,
    ) -> ConversationResult<ConversationModel> {
        match self
            .conversation
            .find_one_and_delete(doc! {"_id" : id})
            .await
        {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ConversationErr::ConversationNotFound),
            Err(err) => Err(ConversationErr::CanNotDoAction {
                action: "delete".to_string(),
                err: err.to_string(),
            }),
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

        let mut cursor = self
            .conversation
            .find(query)
            .sort(doc! {"create_on" : -1})
            .await
            .map_err(|err| ConversationErr::CanNotGetAllByField {
                err: err.to_string(),
                field: field.to_string(),
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

    pub async fn update_conversation_by_id(
        &self,
        id: ObjectId,
        data: Option<ConversationModelPut>,
        add_members: Option<Vec<String>>,
        remove_members: Option<Vec<String>>,
    ) -> ConversationResult<ConversationModel> {
        if let Err(err) = self
            .conversation
            .update_one(
                doc! {"_id" : id, "mms": {"$exists" : false}},
                doc! {"$set" : {"mms" : []}},
            )
            .await
        {
            return Err(ConversationErr::CanNotDoAction {
                err: err.to_string(),
                action: "update".to_string(),
            });
        }

        let mut update_doc = doc! {"$currentDate" : {"uo" : true}};

        if let Some(data) = data {
            update_doc.insert("$set", ConversationModel::put(data));
        }

        if let Some(add) = add_members {
            let members_id: Vec<ObjectId> = add
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();

            if !members_id.is_empty() {
                update_doc.insert(
                    "$addToSet",
                    doc! {
                        "mms": { "$each": members_id }
                    },
                );
            }
        }

        if let Some(remove) = remove_members {
            let members_id: Vec<ObjectId> = remove
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();

            if !members_id.is_empty() {
                update_doc.insert(
                    "$pullAll",
                    doc! {
                        "mms": members_id
                    },
                );
            }
        }

        match self
            .conversation
            .find_one_and_update(doc! {"_id": id}, update_doc)
            .await
        {
            Err(err) => Err(ConversationErr::CanNotDoAction {
                err: err.to_string(),
                action: "update".to_string(),
            }),
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ConversationErr::ConversationNotFound),
        }
    }
}
