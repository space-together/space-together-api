pub type UserRoleResult<T> = core::result::Result<T, UserRoleError>;

#[derive(Debug)]
pub enum UserRoleError {
    CanNotCreateUserRole { err: String },
    RoleIsReadyExit,
    RoleNotFound,
    CanNotFindUserRole { err: String },
    InvalidId,
}

impl std::fmt::Display for UserRoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRoleError::CanNotCreateUserRole { err } => {
                write!(
                    f,
                    "Can't create user role bcs : 😡 {} 😡, try again later",
                    err
                )
            }
            UserRoleError::RoleIsReadyExit => write!(f, "Role is ready to exit, try other role"),
            UserRoleError::RoleNotFound => write!(f, "UserRole not found"),
            UserRoleError::InvalidId => write!(f, "Invalid id"),
            UserRoleError::CanNotFindUserRole { err } => {
                write!(f, "Can't find user role bcs {}", err)
            }
        }
    }
}
