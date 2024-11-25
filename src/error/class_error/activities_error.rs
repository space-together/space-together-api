pub type ActivitiesResult<T> = core::result::Result<T, ActivitiesErr>;

#[derive(Debug)]
pub enum ActivitiesErr {
    Invalid,
    InvalidDateTime,
    CanCreateActivity { error: String },
    CanNotFindActivity { error: String },
    CanGetAllActivity { error: String, field: String },
    CanNotDeleteActivity { error: String },
    CanNotDoAction { error: String, action: String },
    ActivityNotFound,
    ActivityIsReadyExit,
    ActivityTypeIsNotExit,
    ClassAndActivityCanNotHaveOneActivity,
}

impl std::fmt::Display for ActivitiesErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivitiesErr::Invalid => write!(f, " invalid id"),
            ActivitiesErr::InvalidDateTime => write!(f, "Invalid DateTime"),
            ActivitiesErr::ClassAndActivityCanNotHaveOneActivity => {
                write!(f, " A class and an activity cannot share the same activity. Please choose one of them.")
            }
            ActivitiesErr::ActivityTypeIsNotExit => {
                write!(f, " Activity type is not exit, please try other one")
            }
            ActivitiesErr::ActivityIsReadyExit => write!(f, " activity is ready to exit"),
            ActivitiesErr::ActivityNotFound => {
                write!(f, " activity not found , please other activity")
            }
            ActivitiesErr::CanGetAllActivity { error, field } => {
                write!(
                    f,
                    " can't get all in {} activity bcs 😡 {} 😡 ",
                    field, error
                )
            }
            ActivitiesErr::CanCreateActivity { error } => {
                write!(f, " can't create activity bcs 😡 {} 😡 ", error)
            }
            ActivitiesErr::CanNotFindActivity { error } => {
                write!(f, " can't get activity bcs 😡 {} 😡 ", error)
            }
            ActivitiesErr::CanNotDeleteActivity { error } => {
                write!(f, " can't delete activity bcs 😡 {} 😡 ", error)
            }
            ActivitiesErr::CanNotDoAction { error, action } => {
                write!(f, " can't do {} activity bcs  😡{} 😡 ", action, error)
            }
        }
    }
}
