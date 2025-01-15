use super::{
    class_db::{
        activities_db::ActivityDb, activities_type_db::ActivitiesTypeDb, class_db_db::ClassDb,
        class_group_db::ClassGroupDb,
    },
    conversation_db::message_db::MessageDb,
    request_db::{request_db_db::RequestDb, request_type_db::RequestTypeDb},
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};
use crate::{
    error::db_error::{DbError, DbResult},
    libs::{
        classes::db_crud::MongoCrud,
        db::{
            conversation_db::conversation_db_db::ConversationDb,
            db_status::db_status_db::get_database_stats,
        },
    },
    models::{
        auth::adapter_model::{AccountModel, SessionModel},
        class_model::{
            class_room_model::ClassRoomModel, class_room_type_model::ClassRoomTypeModel,
            class_type_model::ClassTypeModel,
        },
        database_model::collection_model::DatabaseStats,
        images_model::{
            profile_images_model::ProfileImageModel, school_logo_model::SchoolLogoModel,
        },
        school_model::{school_model_model::SchoolModel, school_section_model::SchoolSectionModel},
        subject_model::{subject_model_model::SubjectModel, subject_type_model::SubjectTypeModel},
    },
};
use dotenv::dotenv;
use mongodb::Client;
use std::env;

#[derive(Debug)]
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: ClassDb,
    pub class_group: ClassGroupDb,
    pub class_room_type: MongoCrud<ClassRoomTypeModel>,
    pub class_room: MongoCrud<ClassRoomModel>,
    pub conversation: ConversationDb,
    pub message: MessageDb,
    pub activities_type: ActivitiesTypeDb,
    pub activity: ActivityDb,
    pub stats: Option<DatabaseStats>,
    pub request_type: RequestTypeDb,
    pub request: RequestDb,
    pub school: MongoCrud<SchoolModel>,
    pub school_section: MongoCrud<SchoolSectionModel>,
    pub class_type: MongoCrud<ClassTypeModel>,
    pub subject_type: MongoCrud<SubjectTypeModel>,
    pub subject: MongoCrud<SubjectModel>,
    // images
    pub avatars: MongoCrud<ProfileImageModel>,
    pub school_logo: MongoCrud<SchoolLogoModel>,
    // auth
    pub session: MongoCrud<SessionModel>,
    pub account: MongoCrud<AccountModel>,
}

impl ConnDb {
    pub async fn init() -> DbResult<Self> {
        dotenv().ok();
        let bd_uri = match env::var("MONGODB_URI") {
            Ok(val) => val.to_string(),
            Err(_) => "mongodb://localhost:27017/".to_string(),
        };

        let client = Client::with_uri_str(bd_uri).await;

        match client {
            Ok(res) => {
                let st_data = res.database("space-together-data");
                let st_image = res.database("space-together-images");

                let stats_result = get_database_stats(&res, "space-together-data").await;
                let stats = match stats_result {
                    Ok(s) => Some(s),
                    Err(_) => None,
                };

                // Initialize collections
                let user_role = UserRoleDb {
                    role: st_data.collection("users.role"),
                };
                let user = UserDb {
                    user: st_data.collection("users"),
                };
                let class = ClassDb {
                    class: st_data.collection("classes"),
                };
                let class_group = ClassGroupDb {
                    class_group: st_data.collection("--classes_groups"), // private
                };

                let class_type = MongoCrud {
                    collection: st_data.collection("class_type.role"),
                };

                let class_room_type = MongoCrud {
                    collection: st_data.collection("class_room_type.role"),
                };

                let class_room = MongoCrud {
                    collection: st_data.collection("class_room"),
                };

                let conversation = ConversationDb {
                    conversation: st_data.collection("--conversations"), // private
                };
                let message = MessageDb {
                    message: st_data.collection("--messages"), // private collection
                };
                let activities_type = ActivitiesTypeDb {
                    activities_type: st_data.collection("classes_activities.role"), // role
                };
                let activity = ActivityDb {
                    activity: st_data.collection("--classes_activities"), //
                };
                let request_type = RequestTypeDb {
                    request: st_data.collection("requests.role"), // role for request
                };
                let request = RequestDb {
                    request: st_data.collection("requests"),
                };

                // schools
                let school = MongoCrud {
                    collection: st_data.collection("schools"),
                };

                // image collections
                let avatars = MongoCrud {
                    collection: st_image.collection("avatars"),
                };

                let school_logo = MongoCrud {
                    collection: st_image.collection("school_logo"),
                };
                let school_section = MongoCrud {
                    collection: st_data.collection("shool_section.role"),
                };

                let subject_type = MongoCrud {
                    collection: st_data.collection("subject_type.role"),
                };

                let subject = MongoCrud {
                    collection: st_data.collection("subjects"),
                };

                // auth model
                let session = MongoCrud {
                    collection: st_data.collection("--session"),
                };
                let account = MongoCrud {
                    collection: st_data.collection("account"),
                };

                println!("Database connected successfully 🌼");

                Ok(Self {
                    user_role,
                    user,
                    class,
                    class_group,
                    conversation,
                    message,
                    activities_type,
                    activity,
                    stats,
                    request_type,
                    request,
                    school,
                    school_section,
                    subject_type,
                    subject,
                    class_type,
                    class_room,
                    class_room_type,
                    // images
                    avatars,
                    school_logo,
                    // auth
                    session,
                    account,
                })
            }
            Err(err) => Err(DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }),
        }
    }
}
