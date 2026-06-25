use std::str::FromStr;

use actix_web::web;
use chrono::{DateTime, Datelike, Utc};
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::Paginated,
        join_school_request::{
            CreateJoinSchoolRequest, JoinRole, JoinSchoolByCode, JoinSchoolRequest,
            JoinSchoolRequestResponseToken, JoinSchoolRequestWithRelations, JoinStatus,
            SendRequestUserType,
        },
        school_staff::{parse_staff_type, SchoolStaff, SchoolStaffType},
        student::{Student, StudentPartial, StudentStatus},
        teacher::{parse_teacher_type, Teacher},
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::CountDoc},
    repositories::user_repo::UserRepo,
    services::{
        class_service::ClassService,
        school_service::SchoolService,
        school_staff_service::{SchoolStaffQuery, SchoolStaffService},
        student_service::StudentService,
        teacher_service::TeacherService,
        user_service::UserService,
    },
    utils::{code::generate_school_registration_number, email::is_valid_email},
    utils::object_id::ObjectId,
};

pub struct JoinSchoolRequestService {
    pub pool: PgPool,
}

impl JoinSchoolRequestService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
    }

    fn new_id() -> String {
        ObjectId::new().to_hex()
    }

    fn id_to_string(id: &IdType) -> Result<String, AppError> {
        Ok(IdType::to_object_id(id)?.to_hex())
    }

    fn parse_oid(raw: &str, field: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(raw).map_err(|e| AppError {
            message: format!("Invalid {} ObjectId-compatible ID: {}", field, e),
        })
    }

    fn role_to_string(role: &JoinRole) -> String {
        role.to_string()
    }

    fn role_from_string(raw: &str) -> JoinRole {
        match raw {
            "Teacher" => JoinRole::Teacher,
            "Student" => JoinRole::Student,
            "Staff" => JoinRole::Staff,
            "Parent" => JoinRole::Parent,
            _ => JoinRole::Student,
        }
    }

    fn status_to_string(status: &JoinStatus) -> String {
        match status {
            JoinStatus::Pending => "Pending",
            JoinStatus::Accepted => "Accepted",
            JoinStatus::Rejected => "Rejected",
            JoinStatus::Expired => "Expired",
            JoinStatus::Cancelled => "Cancelled",
        }.to_string()
    }

    fn status_from_string(raw: &str) -> JoinStatus {
        match raw {
            "Accepted" => JoinStatus::Accepted,
            "Rejected" => JoinStatus::Rejected,
            "Expired" => JoinStatus::Expired,
            "Cancelled" => JoinStatus::Cancelled,
            _ => JoinStatus::Pending,
        }
    }

    fn row_to_request(row: PgRow) -> Result<JoinSchoolRequest, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let school_id_str: String = row.try_get("school_id").map_err(Self::db_error)?;
        let sent_by_str: String = row.try_get("sent_by").map_err(Self::db_error)?;
        let role_str: String = row.try_get("role").map_err(Self::db_error)?;
        let status_str: String = row.try_get("status").map_err(Self::db_error)?;

        Ok(JoinSchoolRequest {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid(&school_id_str, "school_id")?,
            invited_user_id: row.try_get::<Option<String>, _>("invited_user_id").ok().flatten()
                .as_deref().map(|s| Self::parse_oid(s, "invited_user_id")).transpose()?,
            class_id: row.try_get::<Option<String>, _>("class_id").ok().flatten()
                .as_deref().map(|s| Self::parse_oid(s, "class_id")).transpose()?,
            role: Self::role_from_string(&role_str),
            email: row.try_get("email").map_err(Self::db_error)?,
            r#type: row.try_get("type").unwrap_or_default(),
            message: row.try_get("message").ok().flatten(),
            status: Self::status_from_string(&status_str),
            sent_at: row.try_get("sent_at").unwrap_or_else(|_| Utc::now()),
            responded_at: row.try_get("responded_at").ok().flatten(),
            expires_at: row.try_get("expires_at").ok().flatten(),
            sent_by: Self::parse_oid(&sent_by_str, "sent_by")?,
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, school_id, invited_user_id, class_id, role, email, type, message, status, \
         sent_at, responded_at, expires_at, sent_by, created_at, updated_at \
         FROM join_school_requests WHERE 1=1"
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(email) LIKE ").push_bind(like)
            .push(" OR lower(type) LIKE ").push_bind(like)
            .push(" OR lower(role) LIKE ").push_bind(like)
            .push(" OR lower(status) LIKE ").push_bind(like)
            .push(" OR lower(coalesce(message,'')) LIKE ").push_bind(like)
            .push(" OR lower(id) LIKE ").push_bind(like)
            .push(")");
    }

    pub async fn create(
        &self,
        request: CreateJoinSchoolRequest,
        sent_by: ObjectId,
        state: &AppState,
    ) -> Result<JoinSchoolRequest, AppError> {
        if let Err(e) = is_valid_email(&request.email) {
            return Err(AppError { message: e });
        }

        if let JoinRole::Student = request.role {
            if request.class_id.is_none() {
                return Err(AppError { message: "Class is required for student join requests".into() });
            }
        }

        let school_id = ObjectId::from_str(&request.school_id).map_err(|e| AppError {
            message: format!("Failed to change school id into objectId:{}", e),
        })?;

        let pending_exists = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM join_school_requests WHERE email = $1 AND school_id = $2 AND status = 'Pending'"
        )
        .bind(&request.email)
        .bind(school_id.to_hex())
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if pending_exists > 0 {
            return Err(AppError { message: "A join request with this email already exists".into() });
        }

        let user_repo = UserRepo::new(&state.pg.pool);
        let user_service = UserService::new(&user_repo);

        let mut invited_user_id = None;
        if let Ok(user) = user_service.get_user_by_email(&request.email).await.map_err(|e| AppError { message: e }) {
            if let Some(schools) = &user.schools {
                if schools.contains(&school_id) {
                    return Err(AppError { message: "User is already a member of this school".into() });
                }
            }
            invited_user_id = user.id;
        }

        let class_id = match request.class_id {
            Some(cid_str) => Some(ObjectId::from_str(&cid_str).map_err(|e| AppError {
                message: format!("Failed to change class id into objectId:{}", e),
            })?),
            None => None,
        };

        let now = Utc::now();
        let id = Self::new_id();

        sqlx::query(
            r#"INSERT INTO join_school_requests
               (id, school_id, invited_user_id, class_id, role, email, type, message, status,
                sent_at, expires_at, sent_by, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)"#,
        )
        .bind(&id)
        .bind(school_id.to_hex())
        .bind(invited_user_id.map(|o| o.to_hex()))
        .bind(class_id.map(|o| o.to_hex()))
        .bind(Self::role_to_string(&request.role))
        .bind(&request.email)
        .bind(&request.r#type)
        .bind(&request.message)
        .bind("Pending")
        .bind(now)
        .bind(now + chrono::Duration::days(7))
        .bind(sent_by.to_hex())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        school_id: Option<&str>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        if let Some(sid) = school_id {
            sql.push(" AND school_id = ").push_bind(sid);
        }
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_request(r),
            None => Err(AppError { message: "Join school request not found".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        school_id: Option<&str>,
    ) -> Result<Paginated<JoinSchoolRequest>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter.as_ref().filter(|v| !v.trim().is_empty()).map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM join_school_requests WHERE 1=1");
        if let Some(sid) = school_id { cq.push(" AND school_id = ").push_bind(sid); }
        if let Some(ref l) = like { Self::push_search(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(sid) = school_id { dq.push(" AND school_id = ").push_bind(sid); }
        if let Some(ref l) = like { Self::push_search(&mut dq, l); }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_request).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn accept_request(
        &self,
        id: &IdType,
        invited_user_id: ObjectId,
        responded_by: Option<ObjectId>,
        state: web::Data<AppState>,
        logged_user: &AuthUserDto,
    ) -> Result<JoinSchoolRequestResponseToken, AppError> {
        let request = self.find_one(Some(id), None).await?;
        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError { message: "Join request is not pending".into() });
        }

        let user_repo = UserRepo::new(&state.pg.pool);
        let user_service = UserService::new(&user_repo);
        let user = user_service.get_user_by_email(&request.email).await.map_err(|e| AppError { message: e })?;

        let _ = self.create_role_entity_school_db(&request, &user, &state).await;

        self.update_status(id, JoinStatus::Accepted, Some(invited_user_id), responded_by).await?;

        let school_service = SchoolService::new(&state.pg.pool);
        let school_token = school_service.create_school_token(&IdType::ObjectId(request.school_id), logged_user, &state).await?;

        Ok(JoinSchoolRequestResponseToken { school_token })
    }

    pub async fn reject_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Rejected, None, responded_by).await
    }

    pub async fn cancel_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Cancelled, None, responded_by).await
    }

    async fn update_status(
        &self,
        id: &IdType,
        status: JoinStatus,
        invited_user_id: Option<ObjectId>,
        _responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let id_str = Self::id_to_string(id)?;
        let now = Utc::now();
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE join_school_requests SET");
        sql.push(" status = ").push_bind(Self::status_to_string(&status));
        sql.push(", responded_at = ").push_bind(now);
        sql.push(", updated_at = ").push_bind(now);
        if let Some(uid) = invited_user_id {
            sql.push(", invited_user_id = ").push_bind(uid.to_hex());
        }
        sql.push(" WHERE id = ").push_bind(&id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one(Some(id), None).await
    }

    pub async fn expire_old_requests(&self) -> Result<u64, AppError> {
        let now = Utc::now();
        let result = sqlx::query(
            "UPDATE join_school_requests SET status = 'Expired', updated_at = $1 \
             WHERE expires_at <= $1 AND status = 'Pending'"
        )
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(result.rows_affected())
    }

    pub async fn cleanup_expired_requests(&self, older_than_days: i64) -> Result<CountDoc, AppError> {
        let cutoff = Utc::now() - chrono::Duration::days(older_than_days);
        let result = sqlx::query(
            "DELETE FROM join_school_requests WHERE status = 'Expired' AND updated_at <= $1"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(CountDoc { count: result.rows_affected() })
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        school_id: Option<&str>,
    ) -> Result<Paginated<JoinSchoolRequestWithRelations>, AppError> {
        let page = self.get_all(filter, limit, skip, school_id).await?;
        let mut data = Vec::with_capacity(page.data.len());
        for req in page.data {
            data.push(self.with_relations(req).await?);
        }
        Ok(Paginated { data, total: page.total, total_pages: page.total_pages, current_page: page.current_page })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        school_id: Option<&str>,
    ) -> Result<JoinSchoolRequestWithRelations, AppError> {
        let req = self.find_one(id, school_id).await?;
        self.with_relations(req).await
    }

    async fn with_relations(&self, req: JoinSchoolRequest) -> Result<JoinSchoolRequestWithRelations, AppError> {
        let school = SchoolService::new(&self.pool)
            .find_one(Some(&IdType::ObjectId(req.school_id)), None)
            .await
            .ok();
        let class = match req.class_id {
            Some(cid) => ClassService::new(&self.pool)
                .find_one(Some(&IdType::ObjectId(cid)), None)
                .await
                .ok(),
            None => None,
        };
        let user_repo = UserRepo::new(&self.pool);
        let user_service = UserService::new(&user_repo);
        let invited_user = match req.invited_user_id {
            Some(uid) => user_service.get_user_by_id(&IdType::ObjectId(uid)).await.ok(),
            None => None,
        };
        let sender = user_service.get_user_by_id(&IdType::ObjectId(req.sent_by)).await.ok();

        Ok(JoinSchoolRequestWithRelations { request: req, school, class, invited_user, sender })
    }

    async fn create_role_entity_school_db(
        &self,
        request: &JoinSchoolRequest,
        user: &crate::domain::user::User,
        state: &web::Data<AppState>,
    ) -> Result<(), AppError> {
        let user_id = user.id.unwrap();
        let school_id = request.school_id;
        let school_service = SchoolService::new(&state.pg.pool);
        let school = school_service.find_one(Some(&IdType::ObjectId(school_id)), None).await?;

        match request.role {
            JoinRole::Student => {
                let student_service = StudentService::new(&state.pg.pool);
                let class_service = ClassService::new(&state.pg.pool);

                if let Some(class_id) = request.class_id {
                    if class_service.find_one(Some(&IdType::ObjectId(class_id)), Some(crate::services::class_service::ClassQuery { school_id: Some(school_id.to_hex()), ..Default::default() })).await.is_err() {
                        return Err(AppError { message: "Class not found in school".into() });
                    }
                }

                if let Ok(student) = student_service.find_by_email_in_school(&user.email, &school_id.to_hex()).await {
                    let update_student = StudentPartial {
                        id: None,
                        user_id: Some(user.id),
                        school_id: Some(school.id),
                        class_id: Some(request.class_id),
                        subclass_id: Some(student.subclass_id),
                        creator_id: Some(student.creator_id),
                        name: Some(student.name),
                        email: Some(student.email),
                        phone: Some(student.phone.or(user.phone.clone())),
                        gender: Some(student.gender.or(user.gender.clone())),
                        image: Some(student.image.or(user.image.clone())),
                        image_id: Some(student.image_id.or(user.image_id.clone())),
                        date_of_birth: Some(student.date_of_birth.or(user.age.clone())),
                        registration_number: Some(student.registration_number.or(generate_school_registration_number(&school))),
                        admission_year: Some(student.admission_year),
                        status: Some(student.status),
                        is_active: Some(student.is_active),
                        tags: Some(student.tags),
                        created_at: None,
                        updated_at: None,
                        deleted_at: None,
                        deleted_by: None,
                    };
                    student_service.update(&IdType::from_object_id(student.id.unwrap()), &update_student).await?;
                } else {
                    let new_student = Student {
                        id: None,
                        user_id: Some(user_id),
                        school_id: Some(school_id),
                        class_id: request.class_id,
                        creator_id: Some(request.sent_by),
                        name: user.name.clone(),
                        email: user.email.clone(),
                        phone: user.phone.clone(),
                        gender: user.gender.clone(),
                        date_of_birth: user.age.clone(),
                        subclass_id: None,
                        image: user.image.clone(),
                        image_id: user.image_id.clone(),
                        registration_number: generate_school_registration_number(&school),
                        admission_year: Some(Utc::now().year()),
                        status: StudentStatus::Active,
                        is_active: false,
                        tags: vec!["join-request".to_string()],
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
                        deleted_by: None,
                    };
                    student_service.create(new_student, None).await?;
                }
            }

            JoinRole::Teacher => {
                let teacher_service = TeacherService::new(&state.pg.pool);
                let teacher_type = parse_teacher_type(&request.r#type);
                let new_teacher = Teacher {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    phone: user.phone.clone(),
                    gender: user.gender.clone(),
                    r#type: teacher_type,
                    class_ids: None,
                    subject_ids: None,
                    is_active: false,
                    image: user.image.clone(),
                    image_id: user.image_id.clone(),
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                teacher_service.create(new_teacher, None).await?;
            }

            JoinRole::Staff => {
                let staff_service = SchoolStaffService::new(&state.pg.pool);
                let staff_type = parse_staff_type(&request.r#type);

                match staff_type {
                    SchoolStaffType::Director => {
                        let count = staff_service.count_staff(None, Some(SchoolStaffQuery { school_id: Some(school_id.to_hex()), field_values: vec![("type".to_string(), "Director".to_string())], ..Default::default() })).await?;
                        if count.count >= 1 { return Err(AppError { message: "This school already has a Director".into() }); }
                    }
                    SchoolStaffType::HeadOfStudies => {
                        let count = staff_service.count_staff(None, Some(SchoolStaffQuery { school_id: Some(school_id.to_hex()), field_values: vec![("type".to_string(), "HeadOfStudies".to_string())], ..Default::default() })).await?;
                        if count.count >= 5 { return Err(AppError { message: "This school already has 5 HeadOfStudies".into() }); }
                    }
                }

                let staff = SchoolStaff {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    image: user.image.clone(),
                    image_id: user.image_id.clone(),
                    r#type: staff_type,
                    is_active: false,
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                staff_service.create(staff, None).await?;
            }

            JoinRole::Parent => {
                return Err(AppError { message: "Parent role is not yet supported".into() });
            }
        }

        Ok(())
    }

    pub async fn join_school_by_code(
        &self,
        request: &JoinSchoolByCode,
        auth_user: &AuthUserDto,
        state: web::Data<AppState>,
    ) -> Result<JoinSchoolRequestResponseToken, AppError> {
        let user_repo = UserRepo::new(&state.pg.pool);
        let user_service = UserService::new(&user_repo);
        let user = user_service.get_user_by_email(&auth_user.email).await.map_err(|e| AppError { message: format!("Failed to find user: {}", e) })?;

        let school_service = SchoolService::new(&state.pg.pool);
        let school = school_service.find_by_code(&request.code).await?;

        let school_id = school.id.clone().ok_or_else(|| AppError { message: "School ID not found".into() })?;
        let user_id = user.id.clone().ok_or_else(|| AppError { message: "User ID not found".into() })?;

        let join_request = JoinSchoolRequest::new(&user, &school_id, &user_id);
        self.create_role_entity_school_db(&join_request, &user, &state).await?;

        user_service.add_school_to_user(&IdType::ObjectId(user_id), &IdType::from_object_id(school_id.clone())).await.map_err(|e| AppError { message: e })?;

        let school_service = SchoolService::new(&state.pg.pool);
        let school_token = school_service.create_school_token(&IdType::ObjectId(school_id), auth_user, &state).await?;

        Ok(JoinSchoolRequestResponseToken { school_token })
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        school_id: Option<&str>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), school_id).await?.total;
        Ok(CountDoc { count: total as u64 })
    }

    pub async fn get_my_pending_request(&self, user_email: &str) -> Result<Paginated<JoinSchoolRequestWithRelations>, AppError> {
        let page = self.get_all(None, None, None, None).await.map(|mut p| {
            p.data.retain(|r| r.email == user_email && matches!(r.status, JoinStatus::Pending));
            p
        })?;

        let mut data = Vec::with_capacity(page.data.len());
        for req in page.data {
            data.push(self.with_relations(req).await?);
        }
        Ok(Paginated { data, total: page.total, total_pages: page.total_pages, current_page: page.current_page })
    }

    pub async fn update_expiration(
        &self,
        id: &IdType,
        expires_at: DateTime<Utc>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let id_str = Self::id_to_string(id)?;
        sqlx::query("UPDATE join_school_requests SET expires_at = $1, updated_at = now() WHERE id = $2")
            .bind(expires_at)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one(Some(id), None).await
    }

    pub async fn send_join_request(
        &self,
        user: SendRequestUserType,
        sent_by: ObjectId,
        school_id: &IdType,
        role: JoinRole,
        r#type: String,
        state: &AppState,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request = CreateJoinSchoolRequest {
            sent_by: IdType::ObjectId(sent_by).as_string(),
            email: match user {
                SendRequestUserType::Parent(p) => p.email,
                SendRequestUserType::Teacher(t) => t.email,
                SendRequestUserType::Student(s) => s.email,
                SendRequestUserType::SchoolStaff(st) => st.email,
            },
            role,
            r#type,
            school_id: school_id.as_string(),
            message: None,
            class_id: None,
        };
        self.create(request, sent_by, state).await
    }
}
