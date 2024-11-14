use std::{collections::HashSet, sync::Arc};

use futures::future::try_join_all;

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

    let find_users = try_join_all(
        conversation
            .mms
            .iter()
            .map(|ids| state.db.user.get_user_by_id(ids.to_string())),
    )
    .await;

    if find_users.is_err() {
        return Err(ConversationErr::ConversationMemberNotFound);
    }

    // Proceed to create conversation
    let create = state
        .db
        .conversation
        .create_conversation(conversation)
        .await;

    match create {
        Ok(res) => {
            // Retrieve and convert the inserted ID to a string format
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ConversationErr::InvalidId)?;

            // Get the conversation by ID
            let get = state.db.conversation.get_conversation_by_id(id).await;
            match get {
                Ok(result) => Ok(ConversationModel::format(result)),
                Err(err) => Err(err),
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
