pub type UserResult<T> = core::result::Result<T, UserError>;

#[derive(Debug)]
pub enum UserError {
    CanNotCreateUser { err: String },
    UserIsReadyExit,
    UserNotFound,
    CanNotFindUser { err: String },
    InvalidId,
    UserRoleIsNotExit,
    EmailIsReadyExit,
    CanNotGetAllUsers { err: String },
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::CanNotCreateUser { err } => {
                write!(f, "Can't create user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::UserIsReadyExit => write!(f, "User is ready to exit, try other user"),
            UserError::UserNotFound => write!(f, "User not found"),
            UserError::CanNotFindUser { err } => {
                write!(f, "Can't find user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::InvalidId => write!(f, "Invalid id"),
            UserError::UserRoleIsNotExit => {
                write!(f, "User's role is not exit, try other user role")
            }
            UserError::EmailIsReadyExit => write!(f, "Email is ready to exit, try other email"),
            UserError::CanNotGetAllUsers { err } => {
                write!(
                    f,
                    "Can't get all users bcs : 😡 {} 😡, try again later",
                    err
                )
            }
        }
    }
}