pub type ConversationResult<T> = core::result::Result<T, ConversationErr>;

#[derive(Debug)]
pub enum ConversationErr {
    InvalidId,
    CanNotCreateConversation { err: String },
    ConversationNotFound,
    CanNotFindConversation { err: String },
    CanNotGetAllConversations { err: String },
    UserNotFound,
    ConversationMemberNotFound,
    CanNotGetAllByField { err: String, field: String },
}

impl std::fmt::Display for ConversationErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationErr::InvalidId => write!(f, "Invalid Id, try other id"),
            ConversationErr::CanNotCreateConversation { err } => {
                write!(f, "Can't create conversation bcs 😡 {} 😡 ", err)
            }
            ConversationErr::CanNotFindConversation { err } => {
                write!(f, "Can't find conversation bcs 😡 {} 😡 ", err)
            }
            ConversationErr::ConversationNotFound => {
                write!(f, "Conversation not found, try other conversation")
            }
            ConversationErr::CanNotGetAllConversations { err } => {
                write!(f, "Can not get all conversations bcs 😡 {} 😡 ", err)
            }
            ConversationErr::UserNotFound => write!(f, "User not found, try other users"),
            ConversationErr::CanNotGetAllByField { err, field } => {
                write!(f, "Can not get conversation by {} bcs 😡{}😡", field, err)
            }
            ConversationErr::ConversationMemberNotFound => {
                write!(f, "Conversation member not found, try other users")
            }
        }
    }
}
