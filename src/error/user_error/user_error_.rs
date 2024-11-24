pub type UserResult<T> = core::result::Result<T, UserError>;

#[derive(Debug)]
pub enum UserError {
    CanNotCreateUser { err: String },
    UserIsReadyExit { field: String, value: String },
    UserNotFound { field: String },
    CanNotFindUser { err: String },
    InvalidId,
    InvalidName { err: String },
    InvalidUsername { err: String },
    UserRoleIsNotExit,
    CanNotGetAllUsers { err: String, field: String },
    CanNotGetRole,
    CanNotDoActionUser { err: String, action: String },
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::CanNotCreateUser { err } => {
                write!(f, "Can't create user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::InvalidName { err } => {
                write!(f, "Invalid name: {}", err)
            }
            UserError::InvalidUsername { err } => {
                write!(f, "Invalid username: {}", err)
            }
            UserError::CanNotGetRole => {
                write!(f, "Can not get user role, please try other user role ")
            }
            UserError::CanNotDoActionUser { err, action } => {
                write!(f, "Can not {} user bcs: 😡 {} 😡", action, err)
            }
            UserError::UserIsReadyExit { field, value } => write!(
                f,
                "{} is ready to exit {}, try other {}",
                field, value, field
            ),
            UserError::UserNotFound { field } => {
                write!(f, "User not found by {}, please try other {}", field, field)
            }
            UserError::CanNotFindUser { err } => {
                write!(f, "Can't find user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::InvalidId => write!(f, "Invalid id"),
            UserError::UserRoleIsNotExit => {
                write!(f, "User's role is not exit, try other user role")
            }
            UserError::CanNotGetAllUsers { err, field } => {
                write!(
                    f,
                    "Can't get users in {} bcs : 😡 {} 😡, try again later",
                    field, err
                )
            }
        }
    }
}
