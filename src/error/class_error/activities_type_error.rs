pub type ActivitiesTypeResult<T> = core::result::Result<T, ActivitiesTypeErr>;

#[derive(Debug)]
pub enum ActivitiesTypeErr {
    InvalidId,
    CanNotCreateActivitiesType { err: String },
    ActivitiesTypeNotFound,
    CanNotFindActivitiesType { err: String },
    CanNotGetAllActivitiesTypes { err: String },
    ActivitiesTypeIsReadyExit,
}

impl std::fmt::Display for ActivitiesTypeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivitiesTypeErr::ActivitiesTypeIsReadyExit => write!(
                f,
                "Activity is ready to exit, please try other activity type name"
            ),
            ActivitiesTypeErr::InvalidId => write!(f, "Invalid id"),
            ActivitiesTypeErr::ActivitiesTypeNotFound => {
                write!(f, "Activity type is not found, try other id")
            }
            ActivitiesTypeErr::CanNotFindActivitiesType { err } => {
                write!(f, "Can not find activity type bcs 😡 {} 😡 ", err)
            }
            ActivitiesTypeErr::CanNotGetAllActivitiesTypes { err } => {
                write!(f, "Can not get all activity types bcs 😡 {} 😡 ", err)
            }
            ActivitiesTypeErr::CanNotCreateActivitiesType { err } => {
                write!(f, "Can't create activity type bcs 😡 {} 😡 ", err)
            }
        }
    }
}
