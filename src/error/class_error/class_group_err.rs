pub type ClassGroupResult<T> = core::result::Result<T, ClassGroupErr>;

#[derive(Debug)]
pub enum ClassGroupErr {
    InvalidId,
    CanNotCreateClassGroup { err: String },
    CanNotGetClassById,
    ClassGroupNotFoundById,
    CanNotFindClassGroup { err: String },
}

impl std::fmt::Display for ClassGroupErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassGroupErr::InvalidId => write!(f, "Invalid Id"),
            ClassGroupErr::CanNotCreateClassGroup { err } => {
                write!(f, "Can't not create class group bcs 😡 {} 😡 ", err)
            }
            ClassGroupErr::CanNotGetClassById => {
                write!(f, "Can't get class by id, please other id")
            }
            ClassGroupErr::ClassGroupNotFoundById => {
                write!(f, "Class group not found, try other id ")
            }
            ClassGroupErr::CanNotFindClassGroup { err } => {
                write!(f, "Can't find class group bcs 😡 {} 😡", err)
            }
        }
    }
}
