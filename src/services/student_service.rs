use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        common_details::{Age, Gender, Paginated},
        student::{Student, StudentPartial, StudentStatus, StudentWithRelations},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    repositories::user_repo::UserRepo,
    services::{cloudinary_service::CloudinaryService, school_service::SchoolService},
    utils::{
        email::is_valid_email,
        names::is_valid_name,
        object_id::{parse_object_id_value, ObjectId},
    },
};

#[derive(Debug, Clone, Default)]
pub struct StudentQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub class_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl StudentQuery {
    pub fn from_request(
        query: &RequestQuery,
        context_school_id: Option<String>,
    ) -> Result<Self, AppError> {
        if query.field.len() != query.value.len() {
            return Err(AppError {
                message: "Number of fields must match number of values".into(),
            });
        }

        for id in &query.by_ids {
            parse_object_id_value(id)?;
        }

        Ok(Self {
            by_ids: query.by_ids.clone(),
            school_id: query.school_id.clone().or(context_school_id),
            class_id: query.class_id.clone(),
            field_values: query
                .field
                .iter()
                .cloned()
                .zip(query.value.iter().cloned())
                .collect(),
        })
    }

    pub fn from_school_context(context_school_id: Option<String>) -> Self {
        Self {
            school_id: context_school_id,
            ..Self::default()
        }
    }
}

pub struct StudentService {
    pub pool: PgPool,
}

impl StudentService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        Ok(())
    }

    fn db_error(error: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", error),
        }
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

    fn parse_oid_opt(raw: Option<String>, field: &str) -> Result<Option<ObjectId>, AppError> {
        raw.map(|value| Self::parse_oid(&value, field)).transpose()
    }

    fn enum_to_string<T: Serialize>(value: &T) -> Option<String> {
        match serde_json::to_value(value).ok()? {
            serde_json::Value::String(value) => Some(value),
            other => Some(other.to_string()),
        }
    }

    fn enum_from_string<T: DeserializeOwned>(value: Option<String>) -> Option<T> {
        value.and_then(|raw| serde_json::from_value(serde_json::Value::String(raw)).ok())
    }

    fn status_to_string(status: &StudentStatus) -> String {
        match status {
            StudentStatus::Active => "Active",
            StudentStatus::Suspended => "Suspended",
            StudentStatus::Graduated => "Graduated",
            StudentStatus::Left => "Left",
        }
        .to_string()
    }

    fn status_from_string(value: Option<String>) -> StudentStatus {
        match value
            .as_deref()
            .unwrap_or("Active")
            .to_ascii_lowercase()
            .as_str()
        {
            "suspended" => StudentStatus::Suspended,
            "graduated" => StudentStatus::Graduated,
            "left" => StudentStatus::Left,
            _ => StudentStatus::Active,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT
          sse.id AS enrollment_id,
          sse.student_id AS profile_id,
          sp.user_id,
          sse.school_id,
          sse.class_id,
          sse.subclass_id,
          sse.creator_id,
          sp.name,
          sp.email,
          sp.phone,
          sp.gender,
          sp.image,
          sp.image_id,
          sp.date_of_birth_year,
          sp.date_of_birth_month,
          sp.date_of_birth_day,
          sse.registration_number,
          sse.admission_year,
          sse.status,
          sse.is_active,
          sse.created_at,
          sse.updated_at,
          sse.deleted_at,
          sse.deleted_by
        FROM student_school_enrollments sse
        JOIN student_profiles sp ON sp.id = sse.student_id
        WHERE sp.deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a StudentQuery>,
    ) -> Result<(), AppError> {
        let Some(query) = query else {
            sql.push(" AND sse.deleted_at IS NULL");
            return Ok(());
        };

        sql.push(" AND sse.deleted_at IS NULL");

        if !query.by_ids.is_empty() {
            sql.push(" AND sse.id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                parse_object_id_value(id)?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }

        if let Some(school_id) = &query.school_id {
            parse_object_id_value(school_id)?;
            sql.push(" AND sse.school_id = ").push_bind(school_id);
        }

        if let Some(class_id) = &query.class_id {
            parse_object_id_value(class_id)?;
            sql.push(" AND sse.class_id = ").push_bind(class_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sse.id = ").push_bind(value);
                }
                "student_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sp.id = ").push_bind(value);
                }
                "user_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sp.user_id = ").push_bind(value);
                }
                "school_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sse.school_id = ").push_bind(value);
                }
                "class_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sse.class_id = ").push_bind(value);
                }
                "subclass_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sse.subclass_id = ").push_bind(value);
                }
                "creator_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND sse.creator_id = ").push_bind(value);
                }
                "email" => {
                    sql.push(" AND lower(coalesce(sp.email, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "name" => {
                    sql.push(" AND lower(sp.name) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "registration_number" => {
                    sql.push(" AND lower(coalesce(sse.registration_number, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "phone" => {
                    sql.push(" AND lower(coalesce(sp.phone, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "gender" => {
                    sql.push(" AND lower(coalesce(sp.gender, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "status" => {
                    sql.push(" AND lower(sse.status) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "is_active" => {
                    let parsed = value.parse::<bool>().map_err(|_| AppError {
                        message: "Invalid is_active filter value".into(),
                    })?;
                    sql.push(" AND sse.is_active = ").push_bind(parsed);
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported student filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(sp.name) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sp.email, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sp.phone, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sp.gender, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sse.registration_number, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(sse.status) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(sse.id) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(sp.id) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sp.user_id, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(sse.school_id) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(sse.class_id, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR EXISTS (SELECT 1 FROM student_enrollment_tags setg WHERE setg.enrollment_id = sse.id AND lower(setg.tag) LIKE ")
            .push_bind(search_like)
            .push("))");
    }

    async fn fetch_tags(&self, enrollment_id: &str) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT tag
            FROM student_enrollment_tags
            WHERE enrollment_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(enrollment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| row.try_get("tag").map_err(Self::db_error))
            .collect()
    }

    async fn sync_tags(&self, enrollment_id: &str, tags: &[String]) -> Result<(), AppError> {
        sqlx::query("DELETE FROM student_enrollment_tags WHERE enrollment_id = $1")
            .bind(enrollment_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, tag) in tags.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO student_enrollment_tags (id, enrollment_id, tag, position)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(Self::new_id())
            .bind(enrollment_id)
            .bind(tag)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn student_from_row(&self, row: PgRow) -> Result<Student, AppError> {
        let enrollment_id: String = row.try_get("enrollment_id").map_err(Self::db_error)?;
        let tags = self.fetch_tags(&enrollment_id).await?;
        let dob = match (
            row.try_get("date_of_birth_year").ok().flatten(),
            row.try_get("date_of_birth_month").ok().flatten(),
            row.try_get("date_of_birth_day").ok().flatten(),
        ) {
            (Some(year), Some(month), Some(day)) => Some(Age { year, month, day }),
            _ => None,
        };

        Ok(Student {
            id: Some(Self::parse_oid(&enrollment_id, "id")?),
            user_id: Self::parse_oid_opt(row.try_get("user_id").ok().flatten(), "user_id")?,
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            subclass_id: Self::parse_oid_opt(
                row.try_get("subclass_id").ok().flatten(),
                "subclass_id",
            )?,
            creator_id: Self::parse_oid_opt(
                row.try_get("creator_id").ok().flatten(),
                "creator_id",
            )?,
            name: row.try_get("name").map_err(Self::db_error)?,
            email: row
                .try_get::<Option<String>, _>("email")
                .ok()
                .flatten()
                .unwrap_or_default(),
            phone: row.try_get("phone").ok().flatten(),
            gender: Self::enum_from_string::<Gender>(row.try_get("gender").ok().flatten()),
            image: row.try_get("image").ok().flatten(),
            image_id: row.try_get("image_id").ok().flatten(),
            date_of_birth: dob,
            registration_number: row.try_get("registration_number").ok().flatten(),
            admission_year: row.try_get("admission_year").ok().flatten(),
            status: Self::status_from_string(row.try_get("status").ok().flatten()),
            is_active: row.try_get("is_active").ok().flatten().unwrap_or(false),
            tags,
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
            deleted_at: row.try_get("deleted_at").ok().flatten(),
            deleted_by: Self::parse_oid_opt(
                row.try_get("deleted_by").ok().flatten(),
                "deleted_by",
            )?,
        })
    }

    async fn find_profile_id_for_student(
        &self,
        student: &Student,
    ) -> Result<Option<String>, AppError> {
        if let Some(user_id) = student.user_id {
            let user_id = user_id.to_hex();
            let existing = sqlx::query_scalar::<_, String>(
                "SELECT id FROM student_profiles WHERE user_id = $1 AND deleted_at IS NULL",
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
            if existing.is_some() {
                return Ok(existing);
            }
        }

        if !student.email.trim().is_empty() {
            let existing = sqlx::query_scalar::<_, String>(
                "SELECT id FROM student_profiles WHERE lower(email) = lower($1) AND deleted_at IS NULL ORDER BY created_at ASC LIMIT 1",
            )
            .bind(&student.email)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
            if existing.is_some() {
                return Ok(existing);
            }
        }

        if let Some(phone) = &student.phone {
            let existing = sqlx::query_scalar::<_, String>(
                "SELECT id FROM student_profiles WHERE phone = $1 AND deleted_at IS NULL ORDER BY created_at ASC LIMIT 1",
            )
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
            if existing.is_some() {
                return Ok(existing);
            }
        }

        Ok(None)
    }

    async fn create_profile(&self, student: &Student) -> Result<String, AppError> {
        let id = Self::new_id();
        let dob = student.date_of_birth.as_ref();

        sqlx::query(
            r#"
            INSERT INTO student_profiles (
              id, user_id, name, email, phone, gender, image, image_id,
              date_of_birth_year, date_of_birth_month, date_of_birth_day,
              created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(&id)
        .bind(student.user_id.as_ref().map(|id| id.to_hex()))
        .bind(&student.name)
        .bind(Some(student.email.clone()))
        .bind(&student.phone)
        .bind(student.gender.as_ref().and_then(Self::enum_to_string))
        .bind(&student.image)
        .bind(&student.image_id)
        .bind(dob.map(|dob| dob.year))
        .bind(dob.map(|dob| dob.month))
        .bind(dob.map(|dob| dob.day))
        .bind(student.created_at)
        .bind(student.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(id)
    }

    async fn profile_school_enrollment(
        &self,
        profile_id: &str,
        school_id: &str,
    ) -> Result<Option<Student>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(Self::select_sql());
        query
            .push(" AND sse.student_id = ")
            .push_bind(profile_id)
            .push(" AND sse.school_id = ")
            .push_bind(school_id)
            .push(" AND sse.deleted_at IS NULL");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.student_from_row(row).await?)),
            None => Ok(None),
        }
    }

    async fn registration_exists(
        &self,
        school_id: &str,
        registration_number: &str,
        except_enrollment_id: Option<&str>,
    ) -> Result<bool, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT count(*)
            FROM student_school_enrollments
            WHERE school_id =
            "#,
        );
        query
            .push_bind(school_id)
            .push(" AND lower(registration_number) = lower(")
            .push_bind(registration_number)
            .push(") AND deleted_at IS NULL");
        if let Some(id) = except_enrollment_id {
            query.push(" AND id <> ").push_bind(id);
        }

        let count: i64 = query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(count > 0)
    }

    async fn sync_student_membership(&self, student: &Student) -> Result<(), AppError> {
        let (Some(user_id), Some(school_id), Some(enrollment_id)) =
            (student.user_id, student.school_id, student.id)
        else {
            return Ok(());
        };

        let user_id = user_id.to_hex();
        let school_id = school_id.to_hex();
        let enrollment_id = enrollment_id.to_hex();

        sqlx::query(
            r#"
            INSERT INTO school_memberships (id, school_id, user_id, school_user_id, member_type, status, joined_at)
            VALUES ($1, $2, $3, $4, 'STUDENT', 'active', now())
            ON CONFLICT (school_id, user_id, member_type)
            DO UPDATE SET school_user_id = EXCLUDED.school_user_id, status = 'active', ended_at = NULL, updated_at = now()
            "#,
        )
        .bind(Self::new_id())
        .bind(&school_id)
        .bind(&user_id)
        .bind(&enrollment_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (id, user_id, school_id, role, scope, starts_at)
            VALUES ($1, $2, $3, 'STUDENT', 'school', now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(Self::new_id())
        .bind(&user_id)
        .bind(&school_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            UPDATE users
            SET current_school_id = coalesce(current_school_id, $1), current_school_user_id = $2, updated_at = now()
            WHERE id = $3
            "#,
        )
        .bind(&school_id)
        .bind(&enrollment_id)
        .bind(&user_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn create(
        &self,
        dto: Student,
        _state: Option<&AppState>,
    ) -> Result<Student, AppError> {
        self.ensure_indexes().await?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;
        is_valid_email(&dto.email).map_err(|e| AppError { message: e })?;

        let school_id = dto.school_id.ok_or_else(|| AppError {
            message: "school_id is required".into(),
        })?;
        let school_id = school_id.to_hex();

        if let Some(reg_num) = &dto.registration_number {
            if self.registration_exists(&school_id, reg_num, None).await? {
                return Err(AppError {
                    message: format!("Registration number already exists: {:?}", reg_num),
                });
            }
        }

        let mut student = dto;
        if let Some(image_data) = student.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            student.image_id = Some(cloud_res.public_id);
            student.image = Some(cloud_res.secure_url);
        }

        let profile_id = match self.find_profile_id_for_student(&student).await? {
            Some(profile_id) => profile_id,
            None => self.create_profile(&student).await?,
        };

        if let Some(existing) = self
            .profile_school_enrollment(&profile_id, &school_id)
            .await?
        {
            return Err(AppError {
                message: format!("Email already exists: {}", existing.email),
            });
        }

        let enrollment_id = student
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO student_school_enrollments (
              id, student_id, school_id, class_id, subclass_id, source_student_id,
              registration_number, admission_year, status, is_active, creator_id,
              created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $1, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&enrollment_id)
        .bind(&profile_id)
        .bind(&school_id)
        .bind(student.class_id.as_ref().map(|id| id.to_hex()))
        .bind(student.subclass_id.as_ref().map(|id| id.to_hex()))
        .bind(&student.registration_number)
        .bind(student.admission_year)
        .bind(Self::status_to_string(&student.status))
        .bind(student.is_active)
        .bind(student.creator_id.as_ref().map(|id| id.to_hex()))
        .bind(student.created_at)
        .bind(student.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_tags(&enrollment_id, &student.tags).await?;

        let created = self
            .find_one(Some(&IdType::from_string(&enrollment_id)), None)
            .await?;
        self.sync_student_membership(&created).await?;

        Ok(created)
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<StudentQuery>,
    ) -> Result<Student, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            let id = Self::id_to_string(id)?;
            sql.push(" AND sse.id = ").push_bind(id);
        }
        sql.push(" ORDER BY sse.updated_at DESC LIMIT 1");

        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(self.student_from_row(row).await?),
            None => Err(AppError {
                message: "Student not found".into(),
            }),
        }
    }

    pub async fn find_by_email_in_school(
        &self,
        email: &str,
        school_id: &str,
    ) -> Result<Student, AppError> {
        self.find_one(
            None,
            Some(StudentQuery {
                school_id: Some(school_id.to_string()),
                field_values: vec![("email".to_string(), email.to_string())],
                ..StudentQuery::default()
            }),
        )
        .await
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<StudentQuery>,
    ) -> Result<Paginated<Student>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT count(*)
            FROM student_school_enrollments sse
            JOIN student_profiles sp ON sp.id = sse.student_id
            WHERE sp.deleted_at IS NULL
            "#,
        );
        Self::push_query_filters(&mut count_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_search(&mut count_query, search_like);
        }

        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data_query = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut data_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_search(&mut data_query, search_like);
        }
        data_query
            .push(" ORDER BY sse.updated_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);

        let rows = data_query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(self.student_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &StudentPartial) -> Result<Student, AppError> {
        if let Some(ref name) = update.name {
            is_valid_name(name).map_err(|e| AppError { message: e })?;
        }
        if let Some(ref email) = update.email {
            is_valid_email(email).map_err(|e| AppError { message: e })?;
        }

        let existing_student = self.find_one(Some(id), None).await?;
        let enrollment_id = Self::id_to_string(id)?;

        let profile_id: String =
            sqlx::query_scalar("SELECT student_id FROM student_school_enrollments WHERE id = $1")
                .bind(&enrollment_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::db_error)?;

        if let Some(email) = &update.email {
            if existing_student.email != *email {
                let existing_profile: Option<String> = sqlx::query_scalar(
                    "SELECT id FROM student_profiles WHERE lower(email) = lower($1) AND id <> $2 AND deleted_at IS NULL LIMIT 1",
                )
                .bind(email)
                .bind(&profile_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(Self::db_error)?;
                if existing_profile.is_some() {
                    return Err(AppError {
                        message: format!("Email already exists: {}", email),
                    });
                }
            }
        }

        if let Some(Some(reg_number)) = &update.registration_number {
            if existing_student.registration_number.as_ref() != Some(reg_number) {
                let school_id = existing_student
                    .school_id
                    .as_ref()
                    .map(|id| id.to_hex())
                    .ok_or_else(|| AppError {
                        message: "school_id is required".into(),
                    })?;
                if self
                    .registration_exists(&school_id, reg_number, Some(&enrollment_id))
                    .await?
                {
                    return Err(AppError {
                        message: format!("Registration number already exists: {:?}", reg_number),
                    });
                }
            }
        }

        let mut update_data = update.clone();
        if let Some(Some(new_image_data)) = update.image.clone() {
            if Some(new_image_data.clone()) != existing_student.image {
                if let Some(old_image_id) = existing_student.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.image_id = Some(Some(cloud_res.public_id));
                update_data.image = Some(Some(cloud_res.secure_url));
            }
        }

        let mut profile_query =
            QueryBuilder::<Postgres>::new("UPDATE student_profiles SET updated_at = now()");
        let mut profile_touched = false;

        macro_rules! set_profile_required {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    profile_query
                        .push(", ")
                        .push($column)
                        .push(" = ")
                        .push_bind(value);
                    profile_touched = true;
                }
            };
        }
        macro_rules! set_profile_optional {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    profile_query
                        .push(", ")
                        .push($column)
                        .push(" = ")
                        .push_bind(value);
                    profile_touched = true;
                }
            };
        }

        set_profile_required!(name, "name");
        set_profile_required!(email, "email");
        set_profile_optional!(phone, "phone");
        set_profile_optional!(image, "image");
        set_profile_optional!(image_id, "image_id");

        if let Some(value) = update_data.user_id {
            profile_query
                .push(", user_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            profile_touched = true;
        }
        if let Some(value) = update_data.gender {
            profile_query
                .push(", gender = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            profile_touched = true;
        }
        if let Some(value) = update_data.date_of_birth {
            profile_query
                .push(", date_of_birth_year = ")
                .push_bind(value.as_ref().map(|dob| dob.year))
                .push(", date_of_birth_month = ")
                .push_bind(value.as_ref().map(|dob| dob.month))
                .push(", date_of_birth_day = ")
                .push_bind(value.as_ref().map(|dob| dob.day));
            profile_touched = true;
        }

        if profile_touched {
            profile_query
                .push(" WHERE id = ")
                .push_bind(&profile_id)
                .push(" AND deleted_at IS NULL");
            profile_query
                .build()
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        let mut enrollment_query = QueryBuilder::<Postgres>::new(
            "UPDATE student_school_enrollments SET updated_at = now()",
        );
        let mut enrollment_touched = false;

        macro_rules! set_enrollment_optional {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    enrollment_query
                        .push(", ")
                        .push($column)
                        .push(" = ")
                        .push_bind(value);
                    enrollment_touched = true;
                }
            };
        }

        set_enrollment_optional!(registration_number, "registration_number");
        set_enrollment_optional!(admission_year, "admission_year");
        set_enrollment_optional!(deleted_at, "deleted_at");

        if let Some(value) = update_data.school_id {
            enrollment_query
                .push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.class_id {
            enrollment_query
                .push(", class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.subclass_id {
            enrollment_query
                .push(", subclass_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.creator_id {
            enrollment_query
                .push(", creator_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.deleted_by {
            enrollment_query
                .push(", deleted_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.status {
            enrollment_query
                .push(", status = ")
                .push_bind(Self::status_to_string(&value));
            enrollment_touched = true;
        }
        if let Some(value) = update_data.is_active {
            enrollment_query.push(", is_active = ").push_bind(value);
            enrollment_touched = true;
        }

        if enrollment_touched {
            enrollment_query
                .push(" WHERE id = ")
                .push_bind(&enrollment_id)
                .push(" AND deleted_at IS NULL");
            enrollment_query
                .build()
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        if let Some(tags) = &update_data.tags {
            self.sync_tags(&enrollment_id, tags).await?;
        }

        if !profile_touched && !enrollment_touched && update_data.tags.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let student = self.find_one(Some(id), None).await?;
        self.sync_student_membership(&student).await?;
        Ok(student)
    }

    pub async fn delete(&self, id: &IdType, user_id: &IdType) -> Result<Student, AppError> {
        let student = self.find_one(Some(id), None).await?;
        let id = Self::id_to_string(id)?;
        let user_id = Self::id_to_string(user_id)?;

        sqlx::query(
            r#"
            UPDATE student_school_enrollments
            SET deleted_at = now(), deleted_by = $1, updated_at = now()
            WHERE id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(student)
    }

    pub async fn restore(&self, id: &IdType) -> Result<Student, AppError> {
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            r#"
            UPDATE student_school_enrollments
            SET deleted_at = NULL, deleted_by = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id_string)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(id), None).await
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<StudentQuery>,
    ) -> Result<Paginated<StudentWithRelations>, AppError> {
        let students = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(students.data.len());

        for student in students.data {
            data.push(self.with_relations(student).await?);
        }

        Ok(Paginated {
            data,
            total: students.total,
            total_pages: students.total_pages,
            current_page: students.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<StudentQuery>,
    ) -> Result<StudentWithRelations, AppError> {
        let student = self.find_one(id, query).await?;
        self.with_relations(student).await
    }

    async fn with_relations(&self, student: Student) -> Result<StudentWithRelations, AppError> {
        let user_repo = UserRepo::new(&self.pool);
        let school_service = SchoolService::new(&self.pool);

        let user = match student.user_id {
            Some(user_id) => user_repo
                .find_by_id(&IdType::from_object_id(user_id))
                .await?
                .or(None),
            None => None,
        };
        let creator = match student.creator_id {
            Some(user_id) => user_repo
                .find_by_id(&IdType::from_object_id(user_id))
                .await?
                .or(None),
            None => None,
        };
        let school = match student.school_id {
            Some(school_id) => school_service
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };

        Ok(StudentWithRelations {
            student,
            user,
            creator,
            school,
            class: None,
            subclass: None,
        })
    }

    pub async fn count_students(
        &self,
        filter: Option<String>,
        query: Option<StudentQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }
}
