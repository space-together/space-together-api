use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;

use crate::{
    controllers::class_controller::class_group_controller::{
        controller_class_group_create, controller_class_group_get_all,
        controller_class_group_update, controller_get_class_group_by_id,
        controller_get_class_groups_by_class, controller_get_class_groups_by_student,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        class_model::class_group_model::class_group_model_model::{
            ClassGroupModelNew, ClassGroupModelPut,
        },
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_create_class_groups(
    state: Data<AppState>,
    group: Json<ClassGroupModelNew>,
) -> impl Responder {
    let create = controller_class_group_create(state.into_inner(), group.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_class_group_get_all(state: Data<AppState>) -> impl Responder {
    let all = controller_class_group_get_all(state.into_inner()).await;
    match all {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_get_class_group_by_id(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_get_class_group_by_id(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_get_class_group_by_class(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_get_class_groups_by_class(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_get_class_group_by_student(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_get_class_groups_by_student(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_class_group_update_by_id(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<ClassGroupModelPut>,
) -> impl Responder {
    let group_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    match controller_class_group_update(
        state.into_inner(),
        group_id,
        Some(request.into_inner()),
        None,
        None,
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddRemoveStudentsModel {
    pub students: Vec<String>,
}

pub async fn handle_class_group_add_students(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<AddRemoveStudentsModel>,
) -> impl Responder {
    let class_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    let students = request.into_inner().students;

    if students.is_empty() {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: "students is empty".to_string(),
        });
    }

    match controller_class_group_update(state.into_inner(), class_id, None, Some(students), None)
        .await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_class_group_remove_students(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<AddRemoveStudentsModel>,
) -> impl Responder {
    let class_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    let student = request.into_inner().students;

    if student.is_empty() {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: "student is empty".to_string(),
        });
    }

    match controller_class_group_update(state.into_inner(), class_id, None, None, Some(student))
        .await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
