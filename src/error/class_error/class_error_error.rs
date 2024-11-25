pub type ClassResult<T> = core::result::Result<T, ClassError>;

#[derive(Debug)]
pub enum ClassError {
    InvalidId,
    CanCreateClass { err: String },
    CanNotGetClass { err: String },
    ClassNotFoundById,
    ClassTeacherIsNotExit { id: String },
    CanNotGetAllClass { err: String },
    CanNotGetAllClassBy { field: String, err: String },
    CanNotUpdateClass { err: String },
}

impl std::fmt::Display for ClassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassError::InvalidId => write!(f, "Invalid ID"),
            ClassError::CanCreateClass { err } => write!(f, "Can create class bcs 😡 {} 😡 ", err),
            ClassError::CanNotGetClass { err } => write!(f, "Can not get class bcs 😡 {} 😡 ", err),
            ClassError::ClassNotFoundById => {
                write!(f, "Class not found by id, please try other id")
            }
            ClassError::CanNotUpdateClass { err } => {
                write!(f, "Class not update class bcs 😡{}😡", err)
            }
            ClassError::ClassTeacherIsNotExit { id } => {
                write!(f, "Class teacher is not exit [{}], please try other id", id)
            }
            ClassError::CanNotGetAllClass { err } => {
                write!(f, "Can not get all classes bcs 😡 {} 😡 ", err)
            }
            ClassError::CanNotGetAllClassBy { err, field } => {
                write!(
                    f,
                    "Can't get class in {} bcs : 😡 {} 😡, try again later",
                    field, err
                )
            }
        }
    }
}
