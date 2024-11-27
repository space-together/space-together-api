use std::{collections::HashSet, str::FromStr, sync::Arc};

use futures::future::try_join_all;
use mongodb::bson::oid::ObjectId;

use crate::{
    error::conversation_error::conversation_error_error::{ConversationErr, ConversationResult},
    models::conversation_model::conversation_model_model::{
        ConversationModel, ConversationModelGet, ConversationModelNew,
    },
    AppState,
};

pub async fn controller_conversation_create(
    state: Arc<AppState>,
    mut conversation: ConversationModelNew,
) -> ConversationResult<ConversationModelGet> {
    conversation.mms = conversation
        .mms
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let find_users = try_join_all(conversation.mms.iter().map(|ids| async {
        let obj_id = ObjectId::from_str(ids).map_err(|_| ConversationErr::InvalidId)?;
        state
            .db
            .user
            .get_user_by_id(obj_id)
            .await
            .map_err(|_| ConversationErr::ConversationMemberNotFound)
    }))
    .await;

    find_users?;

    let create_result = state
        .db
        .conversation
        .create_conversation(conversation)
        .await;

    match create_result {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .ok_or(ConversationErr::InvalidId)?
                .to_hex();

            // Fetch the newly created conversation
            match state.db.conversation.get_conversation_by_id(id).await {
                Ok(result) => Ok(ConversationModel::format(result)),
                Err(_) => Err(ConversationErr::ConversationNotFound),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_conversation_by_id(
    state: Arc<AppState>,
    id: String,
) -> ConversationResult<ConversationModelGet> {
    let get = state.db.conversation.get_conversation_by_id(id).await;
    match get {
        Ok(res) => Ok(ConversationModel::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_conversation_by_member(
    state: Arc<AppState>,
    id: ObjectId,
) -> ConversationResult<Vec<ConversationModelGet>> {
    let get = state.db.conversation.get_conversation_by_member(id).await;
    match get {
        Ok(res) => Ok(res.into_iter().map(ConversationModel::format).collect()),
        Err(err) => Err(err),
    }
}
