use std::{str::FromStr, sync::Arc};

use mongodb::bson::oid::ObjectId;

use crate::{
    error::class_error::class_error_error::{ClassError, ClassResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::class_model::class_model_model::{ClassModelGet, ClassModelNew, ClassModelPut},
    AppState,
};

pub async fn controller_create_class(
    state: Arc<AppState>,
    class: ClassModelNew,
) -> ClassResult<ClassModelGet> {
    let class_teacher_id = match ObjectId::from_str(&class.class_teacher_id) {
        Err(_) => return Err(ClassError::InvalidId),
        Ok(i) => i,
    };

    let find_user = state.db.user.get_user_by_id(class_teacher_id).await;

    if find_user.is_err() {
        return Err(ClassError::ClassTeacherIsNotExit {
            id: class.class_teacher_id.clone(),
        });
    }

    if let Ok(u) = find_user {
        if let Some(i) = u.role {
            let get_user_role = state.db.user_role.get_user_role_by_id(i).await;
            if let Ok(r) = get_user_role {
                if r.role != *"Teacher" || r.role != *"School Staff" {
                    return Err(ClassError::OtherError {
                        err: "Can't create class because your role is not allowed to create class"
                            .to_string(),
                    });
                };
            }
        } else {
            return Err(ClassError::OtherError {
                err: "To create school you must have role".to_string(),
            });
        }
    }

    match state.db.class.create_class(class).await {
        Ok(id) => {
            let get = state
                .db
                .class
                .get_class_by_id(change_insertoneresult_into_object_id(id))
                .await;
            match get {
                Ok(res) => Ok(ClassModelGet::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_get_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ClassResult<ClassModelGet> {
    let get = state.db.class.get_class_by_id(id).await;
    match get {
        Ok(res) => Ok(ClassModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_class_delete_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> ClassResult<ClassModelGet> {
    match state.db.class.delete_class_by_id(id).await {
        Ok(res) => Ok(ClassModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_classes(state: Arc<AppState>) -> ClassResult<Vec<ClassModelGet>> {
    let all_classes = state.db.class.get_all_classes().await;
    match all_classes {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn controller_class_gets_by_teacher(
    state: Arc<AppState>,
    teacher: ObjectId,
) -> ClassResult<Vec<ClassModelGet>> {
    match state.db.class.get_class_by_teacher(teacher).await {
        Ok(res) => Ok(res.into_iter().map(ClassModelGet::format).collect()),
        Err(err) => Err(err),
    }
}

pub async fn controller_class_gets_by_student(
    state: Arc<AppState>,
    teacher: ObjectId,
) -> ClassResult<Vec<ClassModelGet>> {
    match state.db.class.get_class_by_student(teacher).await {
        Ok(res) => Ok(res.into_iter().map(ClassModelGet::format).collect()),
        Err(err) => Err(err),
    }
}

pub async fn controller_class_update(
    state: Arc<AppState>,
    id: ObjectId,
    class: Option<ClassModelPut>,
    add_students: Option<Vec<String>>,
    remove_students: Option<Vec<String>>,
) -> ClassResult<ClassModelGet> {
    if let Some(class_data) = &class {
        if let Some(class_teacher) = &class_data.class_teacher_id {
            if ObjectId::from_str(class_teacher).is_err() {
                return Err(ClassError::InvalidId);
            }
            if state
                .db
                .user
                .get_user_by_id(ObjectId::from_str(class_teacher).unwrap())
                .await
                .is_err()
            {
                return Err(ClassError::ClassTeacherIsNotExit {
                    id: class_teacher.to_string(),
                });
            }
        }
    }

    match state
        .db
        .class
        .update_class(class, id, add_students, remove_students)
        .await
    {
        Ok(res) => match state.db.class.get_class_by_id(res.id.unwrap()).await {
            Ok(data) => Ok(ClassModelGet::format(data)),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}
