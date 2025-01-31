pub type UserResult<T> = core::result::Result<T, UserError>;

#[derive(Debug)]
pub enum UserError {
    CanNotCreateUser { err: String },
    UserIsReadyExit { field: String, value: String },
    UserNotFound { field: String, value: String },
    CanNotFindUser { err: String },
    InvalidId,
    InvalidUserRoleId,
    InvalidName { err: String },
    InvalidUsername { err: String },
    UserRoleIsNotExit,
    CanNotGetAllUsers { err: String, field: String },
    CanNotGetRole { error: String },
    CanNotDoActionUser { err: String, action: String },
    SomeError { err: String },
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::SomeError { err } => {
                write!(f, "{err}")
            }
            UserError::CanNotCreateUser { err } => {
                write!(f, "Can't create user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::InvalidName { err } => {
                write!(f, "Invalid name: {}", err)
            }
            UserError::InvalidUsername { err } => {
                write!(f, "Invalid username: {}", err)
            }
            UserError::CanNotGetRole { error } => {
                write!(f, "Can not get user role, bcs: 😡 {} 😡", error)
            }
            UserError::CanNotDoActionUser { err, action } => {
                write!(f, "Can not {} user bcs: 😡 {} 😡", action, err)
            }
            UserError::UserIsReadyExit { field, value } => write!(
                f,
                "{} is ready to exit [{}], try other {}",
                field, value, field
            ),
            UserError::UserNotFound { field, value } => {
                write!(
                    f,
                    "{} not found [{}], please try other {}",
                    field, value, field
                )
            }
            UserError::CanNotFindUser { err } => {
                write!(f, "Can't find user bcs : 😡 {} 😡, try again later", err)
            }
            UserError::InvalidId => write!(f, "Invalid id, please try other id"),
            UserError::InvalidUserRoleId => write!(f, "Invalid user role id, please try other id"),
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
