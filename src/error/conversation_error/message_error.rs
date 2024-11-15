pub type MessageResult<T> = core::result::Result<T, MessageError>;

#[derive(Debug)]
pub enum MessageError {
    InvalidId,
    CanNotCreateMessage { err: String },
    MessageNotFound,
    CanNotFindMessage { err: String },
    CanNotGetAllMessagesForConversation { err: String },
    CanNotGetMessageById { err: String },
    CanNotDeleteMessage { err: String },
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::InvalidId => write!(f, "Invalid Id"),
            MessageError::CanNotCreateMessage { err } => {
                write!(f, "Can not create message bcs 😡 {} 😡 ", err)
            }
            MessageError::CanNotFindMessage { err } => {
                write!(f, "Can not find message bcs 😡 {} 😡 ", err)
            }
            MessageError::CanNotGetAllMessagesForConversation { err } => write!(
                f,
                "Can not get all messages conversation bcs 😡 {} 😡 ",
                err
            ),
            MessageError::MessageNotFound => write!(f, "Message not found "),
            MessageError::CanNotGetMessageById { err } => {
                write!(f, "Can not get message by id  😡 {} 😡", err)
            }
            MessageError::CanNotDeleteMessage { err } => {
                write!(f, "Can not delete message bcs 😡 {} 😡 ", err)
            }
        }
    }
}
