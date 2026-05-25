use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        assignment::{Assignment, AssignmentPartial, Submission, SubmissionPartial},
        auth_user::AuthUserDto,
        common_details::UserRole,
    },
    guards::role_guard::{require_parent_child_access, require_permission},
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        assignment_service::{AssignmentQuery, AssignmentService},
        event_service::EventService,
        parent_service::ParentService,
        role_service::RoleService,
    },
    utils::{
        object_id::parse_object_id_value,
        request_context::{postgres_pool, request_context},
    },
};

#[get("")]
async fn get_all_assignments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let assignment_query = match AssignmentQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all_assignments_with_relations(
            query.filter.clone(),
            query.limit,
            query.skip,
            assignment_query,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_assignment_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .find_one_assignment_with_relations(
            Some(&id),
            Some(AssignmentQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Assignment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let school_id = match get_school_id_from_request(&req)
        .or(context.school_id.clone())
        .or_else(|| user.current_school_id.clone())
    {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID required"
            }));
        }
    };

    match service
        .is_feature_enabled(&school_id, "assignments.enabled")
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "Feature assignments.enabled is not enabled"
            }));
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    }

    let role_service = RoleService::new(&state.pg.pool);
    if let Err(e) = require_permission(&user, &school_id, "assignment.create", &role_service).await
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    let mut assignment = data.into_inner();
    if assignment.school_id.is_none() {
        assignment.school_id = match parse_object_id_value(&school_id) {
            Ok(id) => Some(id),
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
    }

    if assignment.teacher_id.is_none() {
        match service
            .find_teacher_id_for_user(&user.id, Some(&school_id))
            .await
        {
            Ok(Some(teacher_id)) => assignment.teacher_id = Some(teacher_id),
            Ok(None) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Teacher record not found for this user"
                }));
            }
            Err(err) => return HttpResponse::BadRequest().json(err),
        }
    }

    match service.create_assignment(assignment).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<AssignmentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));
    let school_id =
        match get_school_id_from_request(&req).or_else(|| user.current_school_id.clone()) {
            Some(id) => id,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "School ID required"
                }));
            }
        };

    let role_service = RoleService::new(&state.pg.pool);
    if let Err(e) = require_permission(&user, &school_id, "assignment.update", &role_service).await
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service.find_one_assignment(Some(&id), None).await {
            Ok(assignment) => {
                if let Some(teacher_id) = assignment.teacher_id {
                    match service.teacher_belongs_to_user(&teacher_id, &user.id).await {
                        Ok(true) => {}
                        _ => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only update your own assignments"
                            }));
                        }
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.update_assignment(&id, &data.into_inner()).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));
    let school_id =
        match get_school_id_from_request(&req).or_else(|| user.current_school_id.clone()) {
            Some(id) => id,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "School ID required"
                }));
            }
        };

    let role_service = RoleService::new(&state.pg.pool);
    if let Err(e) = require_permission(&user, &school_id, "assignment.delete", &role_service).await
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service.find_one_assignment(Some(&id), None).await {
            Ok(assignment) => {
                if let Some(teacher_id) = assignment.teacher_id {
                    match service.teacher_belongs_to_user(&teacher_id, &user.id).await {
                        Ok(true) => {}
                        _ => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only delete your own assignments"
                            }));
                        }
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.delete_assignment(&id).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_assignments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let assignment_query = match AssignmentQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .count_assignments(query.filter.clone(), assignment_query)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/submit")]
async fn submit_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<Submission>,
    state: web::Data<AppState>,
) -> impl Responder {
    if !matches!(user.role, Some(UserRole::STUDENT)) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Only students can submit assignments"
        }));
    }

    let assignment_id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let mut submission = data.into_inner();

    match IdType::to_object_id(&assignment_id) {
        Ok(oid) => submission.assignment_id = Some(oid),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid assignment ID"
            }));
        }
    }

    let student = match service
        .find_student_for_user(&user.id, context.school_id.as_deref())
        .await
    {
        Ok(Some(student)) => student,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Student record not found for this user"
            }));
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    };
    submission.student_id = Some(student.student_id);

    if let Ok(assignment) = service
        .find_one_assignment(Some(&assignment_id), None)
        .await
    {
        if let (Some(assignment_class_id), Some(student_class_id)) =
            (assignment.class_id, student.class_id)
        {
            if assignment_class_id != student_class_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "message": "You are not enrolled in this class"
                }));
            }
        }
    }

    match service.create_submission(submission).await {
        Ok(submission) => {
            let submission_clone = submission.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = submission_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "submission",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &submission_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(submission)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/submissions")]
async fn get_assignment_submissions(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    if !matches!(
        user.role,
        Some(UserRole::TEACHER) | Some(UserRole::ADMIN) | Some(UserRole::SCHOOLSTAFF)
    ) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Insufficient permissions"
        }));
    }

    let assignment_id = match IdType::to_object_id(&IdType::from_string(path.into_inner())) {
        Ok(id) => id.to_hex(),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid assignment ID"
            }));
        }
    };
    let service = AssignmentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let mut submission_query = match AssignmentQuery::from_request(&query, context.school_id) {
        Ok(query) => query,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };
    submission_query.assignment_id = Some(assignment_id);

    match service
        .get_all_submissions_with_relations(
            query.filter.clone(),
            query.limit,
            query.skip,
            Some(submission_query),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{assignment_id}/grade/{submission_id}")]
async fn grade_submission(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, String)>,
    data: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> impl Responder {
    if !matches!(user.role, Some(UserRole::TEACHER) | Some(UserRole::ADMIN)) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Only teachers can grade submissions"
        }));
    }

    let (assignment_id_str, submission_id_str) = path.into_inner();
    let assignment_id = IdType::from_string(assignment_id_str);
    let submission_id = IdType::from_string(submission_id_str);
    let service = AssignmentService::new(postgres_pool(&state));

    let graded_by = match service
        .find_teacher_id_for_user(&user.id, user.current_school_id.as_deref())
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) if matches!(user.role, Some(UserRole::ADMIN)) => {
            match parse_object_id_value(&user.id) {
                Ok(id) => id,
                Err(err) => return HttpResponse::BadRequest().json(err),
            }
        }
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Teacher record not found"
            }));
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service
            .find_one_assignment(Some(&assignment_id), None)
            .await
        {
            Ok(assignment) => {
                if let Some(teacher_id) = assignment.teacher_id {
                    match service.teacher_belongs_to_user(&teacher_id, &user.id).await {
                        Ok(true) => {}
                        _ => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only grade assignments for subjects you teach"
                            }));
                        }
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    let score = match data.get("score").and_then(|v| v.as_f64()) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Score is required"
            }));
        }
    };
    let feedback = data
        .get("feedback")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let feedback_file = data
        .get("feedback_file")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match service
        .grade_submission(&submission_id, score, feedback, feedback_file, graded_by)
        .await
    {
        Ok(submission) => HttpResponse::Ok().json(submission),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/submissions/{id}")]
async fn get_submission_by_id(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));
    let submission = match service.find_one_submission(Some(&id), None).await {
        Ok(sub) => sub,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    if matches!(user.role, Some(UserRole::STUDENT)) {
        if let Some(student_id) = submission.student_id {
            match service.student_belongs_to_user(&student_id, &user.id).await {
                Ok(true) => {}
                _ => {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "message": "You can only view your own submissions"
                    }));
                }
            }
        }
    }

    if matches!(user.role, Some(UserRole::PARENT)) {
        if let Some(student_id) = submission.student_id {
            let parent_service = ParentService::new(&state.pg.pool);
            if let Err(e) =
                require_parent_child_access(&user, &student_id.to_hex(), &parent_service).await
            {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "message": e
                }));
            }
        } else {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "Cannot verify parent access: student ID not found"
            }));
        }
    }

    let submission_id = match IdType::to_object_id(&id) {
        Ok(id) => id.to_hex(),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid ID"
            }));
        }
    };
    let submission_query = AssignmentQuery {
        by_ids: vec![submission_id],
        ..AssignmentQuery::default()
    };

    match service
        .get_all_submissions_with_relations(None, Some(1), None, Some(submission_query))
        .await
    {
        Ok(mut data) => {
            if let Some(submission) = data.data.pop() {
                HttpResponse::Ok().json(submission)
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "message": "Submission not found"
                }))
            }
        }
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[put("/submissions/{id}")]
async fn update_submission(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SubmissionPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AssignmentService::new(postgres_pool(&state));

    if matches!(user.role, Some(UserRole::STUDENT)) {
        match service.find_one_submission(Some(&id), None).await {
            Ok(submission) => {
                if matches!(
                    submission.status,
                    crate::domain::assignment::SubmissionStatus::Graded
                ) {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "message": "Cannot update a graded submission"
                    }));
                }

                if let Some(student_id) = submission.student_id {
                    match service.student_belongs_to_user(&student_id, &user.id).await {
                        Ok(true) => {}
                        _ => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only update your own submissions"
                            }));
                        }
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.update_submission(&id, &data.into_inner()).await {
        Ok(submission) => HttpResponse::Ok().json(submission),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_assignments)
        .service(get_assignment_by_id)
        .service(count_assignments)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_assignment)
                .service(update_assignment)
                .service(delete_assignment)
                .service(submit_assignment)
                .service(get_assignment_submissions)
                .service(grade_submission)
                .service(get_submission_by_id)
                .service(update_submission),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "assignments", blueprint);
}
