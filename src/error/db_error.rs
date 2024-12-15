pub type DbResult<T> = core::result::Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    CanNotConnectToDatabase { err: String },
    CanNotGetAllTables { err: String },
    QueryFailed { err: String },
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::CanNotConnectToDatabase { err } => {
                write!(
                    f,
                    "Can not connect to database bcs : 😡 {} 😡 , try again later",
                    err
                )
            }
            DbError::QueryFailed { err } => {
                write!(f, "Query failed with error 😡 {} 😡", err)
            }
            DbError::CanNotGetAllTables { err } => {
                write!(
                    f,
                    "Can not get tables in database bcs : 😡 {} 😡 , try again later",
                    err
                )
            }
        }
    }
}
