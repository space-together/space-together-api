pub type RequestRequest<T> = core::result::Result<T, RequestError>;

pub enum RequestError {
    CanNotGetRequestByField { field: String, value: String },
    InvalidId,
    CanDoAction { action: String, error: String },
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::InvalidId => write!(f, "Invalid Id, please use MongoId instead"),
            RequestError::CanNotGetRequestByField { field, value } => {
                write!(f, "Can't get request by field {} value {}", field, value)
            }
            RequestError::CanDoAction { action, error } => {
                write!(f, "Can do {} bsc 😡 {} 😡 ", action, error)
            }
        }
    }
}
