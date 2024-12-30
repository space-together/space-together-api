use mongodb::error::Error as MongoError;

pub type DbClassResult<T> = core::result::Result<T, DbClassError>;

#[derive(Debug)]
pub enum DbClassError {
    InvalidId,
    CanNotDoAction {
        error: String,
        action: String,
        how_fix_it: String,
        collection: String,
    },
    OtherError {
        err: String,
    },
}

impl std::fmt::Display for DbClassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbClassError::OtherError { err } => write!(f, "{err}"),
            DbClassError::InvalidId => write!(f, "Invalid id, please try other id"),
            DbClassError::CanNotDoAction {
                error,
                action,
                how_fix_it,
                collection,
            } => {
                write!(
                    f,
                    "Can't {} in {} bcs: 😡 {} 😡, {}",
                    action, collection, error, how_fix_it
                )
            }
        }
    }
}

impl From<MongoError> for DbClassError {
    fn from(err: MongoError) -> Self {
        DbClassError::CanNotDoAction {
            error: err.to_string(),
            collection: " ".to_string(),
            action: "perform database operation".to_string(),
            how_fix_it: "Check database connection, query, and document structure".to_string(),
        }
    }
}
