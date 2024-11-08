pub type UserRoleResult<T> = core::result::Result<T, UserRoleError>;

#[derive(Debug)]
pub enum UserRoleError {
    CanNotCreateUserRole { err: String },
    RoleIsReadyExit,
}

impl std::fmt::Display for UserRoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRoleError::CanNotCreateUserRole { err } => {
                write!(f, "Can't create user role bcs : {}", err)
            }
            UserRoleError::RoleIsReadyExit => write!(f, "Role is ready to exit, try other role"),
        }
    }
}
