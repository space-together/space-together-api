use std::sync::Arc;

use crate::{
    error::conversation_error::message_error::{MessageError, MessageResult},
    models::conversation_model::message_model::{MessageModel, MessageModelGet, MessageModelNew},
    AppState,
};

pub async fn controller_message_create(
    state: Arc<AppState>,
    message: MessageModelNew,
) -> MessageResult<MessageModelGet> {
    let create = state.db.message.create_message(message).await;

    match create {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(MessageError::InvalidId)?;

            let get = state.db.message.get_message_by_id(id).await;
            match get {
                Ok(res) => Ok(MessageModel::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
