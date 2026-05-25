use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        common_details::{Gender, Paginated},
        teacher::{Teacher, TeacherType, TeacherWithRelations, UpdateTeacher},
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
pub struct TeacherQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl TeacherQuery {
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

pub struct TeacherService {
    pub pool: PgPool,
}

impl TeacherService {
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

    fn teacher_type_to_string(value: &TeacherType) -> String {
        value.to_string()
    }

    fn teacher_type_from_string(value: Option<String>) -> TeacherType {
        match value
            .as_deref()
            .unwrap_or("Regular")
            .to_ascii_lowercase()
            .as_str()
        {
            "headteacher" => TeacherType::HeadTeacher,
            "subjectteacher" => TeacherType::SubjectTeacher,
            "deputy" => TeacherType::Deputy,
            _ => TeacherType::Regular,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, user_id, creator_id, name, email, phone, gender, image, image_id,
               teacher_type, is_active, created_at, updated_at
        FROM teachers
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a TeacherQuery>,
    ) -> Result<(), AppError> {
        let Some(query) = query else {
            return Ok(());
        };

        if !query.by_ids.is_empty() {
            sql.push(" AND id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                parse_object_id_value(id)?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }

        if let Some(school_id) = &query.school_id {
            parse_object_id_value(school_id)?;
            sql.push(" AND school_id = ").push_bind(school_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "user_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND user_id = ").push_bind(value);
                }
                "school_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND school_id = ").push_bind(value);
                }
                "creator_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND creator_id = ").push_bind(value);
                }
                "class_id" | "class_ids" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND EXISTS (SELECT 1 FROM teacher_classes tc WHERE tc.teacher_id = teachers.id AND tc.class_id = ")
                        .push_bind(value)
                        .push(")");
                }
                "subject_id" | "subject_ids" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND EXISTS (SELECT 1 FROM teacher_subjects ts WHERE ts.teacher_id = teachers.id AND ts.class_subject_id = ")
                        .push_bind(value)
                        .push(")");
                }
                "email" => {
                    sql.push(" AND lower(coalesce(email, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "name" => {
                    sql.push(" AND lower(coalesce(name, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "phone" => {
                    sql.push(" AND lower(coalesce(phone, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "gender" => {
                    sql.push(" AND lower(coalesce(gender, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "type" => {
                    sql.push(" AND lower(teacher_type) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "is_active" => {
                    let parsed = value.parse::<bool>().map_err(|_| AppError {
                        message: "Invalid is_active filter value".into(),
                    })?;
                    sql.push(" AND is_active = ").push_bind(parsed);
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported teacher filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(email, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(phone, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(gender, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(teacher_type) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(" OR EXISTS (SELECT 1 FROM teacher_tags tt WHERE tt.teacher_id = teachers.id AND lower(tt.tag) LIKE ")
            .push_bind(search_like)
            .push("))");
    }

    async fn fetch_id_relation(
        &self,
        table: &str,
        id_column: &str,
        teacher_id: &str,
    ) -> Result<Option<Vec<ObjectId>>, AppError> {
        let sql = format!(
            "SELECT {} FROM {} WHERE teacher_id = $1 ORDER BY position ASC, created_at ASC",
            id_column, table
        );
        let rows = sqlx::query(&sql)
            .bind(teacher_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get(id_column).map_err(Self::db_error)?;
                Self::parse_oid(&id, id_column)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn fetch_tags(&self, teacher_id: &str) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT tag
            FROM teacher_tags
            WHERE teacher_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(teacher_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| row.try_get("tag").map_err(Self::db_error))
            .collect()
    }

    async fn sync_tags(&self, teacher_id: &str, tags: &[String]) -> Result<(), AppError> {
        sqlx::query("DELETE FROM teacher_tags WHERE teacher_id = $1")
            .bind(teacher_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, tag) in tags.iter().enumerate() {
            sqlx::query(
                "INSERT INTO teacher_tags (id, teacher_id, tag, position) VALUES ($1, $2, $3, $4)",
            )
            .bind(Self::new_id())
            .bind(teacher_id)
            .bind(tag)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn sync_id_relation(
        &self,
        table: &str,
        id_column: &str,
        teacher_id: &str,
        school_id: &str,
        ids: &[ObjectId],
    ) -> Result<(), AppError> {
        let delete_sql = format!("DELETE FROM {} WHERE teacher_id = $1", table);
        sqlx::query(&delete_sql)
            .bind(teacher_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let insert_sql = format!(
            "INSERT INTO {} (id, teacher_id, {}, school_id, position) VALUES ($1, $2, $3, $4, $5)",
            table, id_column
        );
        for (position, id) in ids.iter().enumerate() {
            sqlx::query(&insert_sql)
                .bind(Self::new_id())
                .bind(teacher_id)
                .bind(id.to_hex())
                .bind(school_id)
                .bind(position as i32)
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn teacher_from_row(&self, row: PgRow) -> Result<Teacher, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Teacher {
            id: Some(Self::parse_oid(&id, "id")?),
            user_id: Self::parse_oid_opt(row.try_get("user_id").ok().flatten(), "user_id")?,
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            creator_id: Self::parse_oid_opt(
                row.try_get("creator_id").ok().flatten(),
                "creator_id",
            )?,
            name: row
                .try_get::<Option<String>, _>("name")
                .ok()
                .flatten()
                .unwrap_or_default(),
            email: row
                .try_get::<Option<String>, _>("email")
                .ok()
                .flatten()
                .unwrap_or_default(),
            phone: row.try_get("phone").ok().flatten(),
            gender: Self::enum_from_string::<Gender>(row.try_get("gender").ok().flatten()),
            image: row.try_get("image").ok().flatten(),
            image_id: row.try_get("image_id").ok().flatten(),
            r#type: Self::teacher_type_from_string(row.try_get("teacher_type").ok().flatten()),
            class_ids: self
                .fetch_id_relation("teacher_classes", "class_id", &id)
                .await?,
            subject_ids: self
                .fetch_id_relation("teacher_subjects", "class_subject_id", &id)
                .await?,
            is_active: row.try_get("is_active").ok().flatten().unwrap_or(true),
            tags: self.fetch_tags(&id).await?,
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    async fn email_exists(
        &self,
        email: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<Teacher>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(Self::select_sql());
        query
            .push(" AND lower(coalesce(email, '')) = lower(")
            .push_bind(email)
            .push(")");
        if let Some(school_id) = school_id {
            query.push(" AND school_id = ").push_bind(school_id);
        }
        if let Some(except_id) = except_id {
            query.push(" AND id <> ").push_bind(except_id);
        }
        query.push(" ORDER BY updated_at DESC LIMIT 1");
        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.teacher_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(
        &self,
        mut dto: Teacher,
        _state: Option<&AppState>,
    ) -> Result<Teacher, AppError> {
        self.ensure_indexes().await?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;
        is_valid_email(&dto.email).map_err(|e| AppError { message: e })?;

        let school_id = dto
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        if let Some(existing) = self
            .email_exists(&dto.email, Some(&school_id), None)
            .await?
        {
            return Err(AppError {
                message: format!("Email already exists: {}", existing.email),
            });
        }

        if let Some(image_data) = dto.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            dto.image_id = Some(cloud_res.public_id);
            dto.image = Some(cloud_res.secure_url);
        }

        let id = dto
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO teachers (
              id, school_id, user_id, creator_id, name, email, phone, gender, image, image_id,
              teacher_type, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(dto.user_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.creator_id.as_ref().map(|id| id.to_hex()))
        .bind(&dto.name)
        .bind(&dto.email)
        .bind(&dto.phone)
        .bind(dto.gender.as_ref().and_then(Self::enum_to_string))
        .bind(&dto.image)
        .bind(&dto.image_id)
        .bind(Self::teacher_type_to_string(&dto.r#type))
        .bind(dto.is_active)
        .bind(dto.created_at)
        .bind(dto.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if let Some(class_ids) = &dto.class_ids {
            self.sync_id_relation("teacher_classes", "class_id", &id, &school_id, class_ids)
                .await?;
        }
        if let Some(subject_ids) = &dto.subject_ids {
            self.sync_id_relation(
                "teacher_subjects",
                "class_subject_id",
                &id,
                &school_id,
                subject_ids,
            )
            .await?;
        }
        self.sync_tags(&id, &dto.tags).await?;

        let teacher = self.find_one(Some(&IdType::from_string(&id)), None).await?;
        self.sync_membership(&teacher).await?;
        Ok(teacher)
    }

    async fn sync_membership(&self, teacher: &Teacher) -> Result<(), AppError> {
        let (Some(teacher_id), Some(user_id), Some(school_id)) =
            (teacher.id, teacher.user_id, teacher.school_id)
        else {
            return Ok(());
        };

        sqlx::query(
            r#"
            INSERT INTO school_memberships (id, school_id, user_id, school_user_id, member_type, status, joined_at)
            VALUES ($1, $2, $3, $4, 'TEACHER', 'active', now())
            ON CONFLICT (school_id, user_id, member_type)
            DO UPDATE SET school_user_id = EXCLUDED.school_user_id, status = 'active', ended_at = NULL, updated_at = now()
            "#,
        )
        .bind(Self::new_id())
        .bind(school_id.to_hex())
        .bind(user_id.to_hex())
        .bind(teacher_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (id, user_id, school_id, role, scope, starts_at)
            VALUES ($1, $2, $3, 'TEACHER', 'school', now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(Self::new_id())
        .bind(user_id.to_hex())
        .bind(school_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<TeacherQuery>,
    ) -> Result<Teacher, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        sql.push(" ORDER BY updated_at DESC LIMIT 1");

        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(self.teacher_from_row(row).await?),
            None => Err(AppError {
                message: "Teacher not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<TeacherQuery>,
    ) -> Result<Paginated<Teacher>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM teachers WHERE deleted_at IS NULL");
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
            .push(" ORDER BY updated_at DESC LIMIT ")
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
            data.push(self.teacher_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &UpdateTeacher) -> Result<Teacher, AppError> {
        if let Some(ref name) = update.name {
            is_valid_name(name).map_err(|e| AppError { message: e })?;
        }
        if let Some(ref email) = update.email {
            is_valid_email(email).map_err(|e| AppError { message: e })?;
        }

        let existing = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        let school_id = existing
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;

        if let Some(ref email) = update.email {
            if existing.email != *email
                && self
                    .email_exists(email, Some(&school_id), Some(&id_string))
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("Email already exists: {}", email),
                });
            }
        }

        let mut update_data = update.clone();
        if let Some(Some(new_image)) = update.image.clone() {
            if Some(new_image.clone()) != existing.image {
                if let Some(old_image_id) = existing.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }
                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image)
                    .await
                    .map_err(|e| AppError { message: e })?;
                update_data.image_id = Some(Some(cloud_res.public_id));
                update_data.image = Some(Some(cloud_res.secure_url));
            }
        }

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE teachers SET updated_at = now()");
        let mut touched = false;

        macro_rules! set_required {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }
        macro_rules! set_optional {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }

        set_required!(name, "name");
        set_required!(email, "email");
        set_optional!(phone, "phone");
        set_optional!(image, "image");
        set_optional!(image_id, "image_id");

        if let Some(value) = update_data.user_id {
            sql.push(", user_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.creator_id {
            sql.push(", creator_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.gender {
            sql.push(", gender = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched = true;
        }
        if let Some(value) = update_data.r#type {
            sql.push(", teacher_type = ")
                .push_bind(Self::teacher_type_to_string(&value));
            touched = true;
        }
        if let Some(value) = update_data.is_active {
            sql.push(", is_active = ").push_bind(value);
            touched = true;
        }

        if touched {
            sql.push(" WHERE id = ")
                .push_bind(&id_string)
                .push(" AND deleted_at IS NULL");
            sql.build()
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        if let Some(Some(class_ids)) = &update_data.class_ids {
            self.sync_id_relation(
                "teacher_classes",
                "class_id",
                &id_string,
                &school_id,
                class_ids,
            )
            .await?;
        }
        if let Some(Some(subject_ids)) = &update_data.subject_ids {
            self.sync_id_relation(
                "teacher_subjects",
                "class_subject_id",
                &id_string,
                &school_id,
                subject_ids,
            )
            .await?;
        }
        if let Some(tags) = &update_data.tags {
            self.sync_tags(&id_string, tags).await?;
        }

        if !touched
            && update_data.class_ids.is_none()
            && update_data.subject_ids.is_none()
            && update_data.tags.is_none()
        {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let teacher = self.find_one(Some(id), None).await?;
        self.sync_membership(&teacher).await?;
        Ok(teacher)
    }

    pub async fn delete(&self, id: &IdType) -> Result<Teacher, AppError> {
        let teacher = self.find_one(Some(id), None).await?;
        let id = Self::id_to_string(id)?;
        if let Some(ref image_id) = teacher.image_id {
            CloudinaryService::delete_from_cloudinary(image_id)
                .await
                .ok();
        }
        sqlx::query("UPDATE teachers SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(teacher)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<TeacherQuery>,
    ) -> Result<Paginated<TeacherWithRelations>, AppError> {
        let teachers = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(teachers.data.len());
        for teacher in teachers.data {
            data.push(self.with_relations(teacher).await?);
        }

        Ok(Paginated {
            data,
            total: teachers.total,
            total_pages: teachers.total_pages,
            current_page: teachers.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<TeacherQuery>,
    ) -> Result<TeacherWithRelations, AppError> {
        let teacher = self.find_one(id, query).await?;
        self.with_relations(teacher).await
    }

    async fn with_relations(&self, teacher: Teacher) -> Result<TeacherWithRelations, AppError> {
        let user_repo = UserRepo::new(&self.pool);
        let school_service = SchoolService::new(&self.pool);

        let user = match teacher.user_id {
            Some(user_id) => {
                user_repo
                    .find_by_id(&IdType::from_object_id(user_id))
                    .await?
            }
            None => None,
        };
        let creator = match teacher.creator_id {
            Some(user_id) => {
                user_repo
                    .find_by_id(&IdType::from_object_id(user_id))
                    .await?
            }
            None => None,
        };
        let school = match teacher.school_id {
            Some(school_id) => school_service
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };

        Ok(TeacherWithRelations {
            teacher,
            user,
            creator,
            school,
            classes: None,
            subjects: None,
        })
    }

    pub async fn count_teachers(
        &self,
        filter: Option<String>,
        query: Option<TeacherQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub async fn add_subjects_to_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let teacher = self.find_one(Some(teacher_id), None).await?;
        let teacher_id = Self::id_to_string(teacher_id)?;
        let school_id = teacher
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let mut ids = teacher.subject_ids.unwrap_or_default();
        for subject_id in subject_ids {
            if !ids.contains(&subject_id) {
                ids.push(subject_id);
            }
        }
        self.sync_id_relation(
            "teacher_subjects",
            "class_subject_id",
            &teacher_id,
            &school_id,
            &ids,
        )
        .await?;
        self.find_one(Some(&IdType::from_string(teacher_id)), None)
            .await
    }

    pub async fn add_classes_to_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let teacher = self.find_one(Some(teacher_id), None).await?;
        let teacher_id = Self::id_to_string(teacher_id)?;
        let school_id = teacher
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let mut ids = teacher.class_ids.unwrap_or_default();
        for class_id in class_ids {
            if !ids.contains(&class_id) {
                ids.push(class_id);
            }
        }
        self.sync_id_relation("teacher_classes", "class_id", &teacher_id, &school_id, &ids)
            .await?;
        self.find_one(Some(&IdType::from_string(teacher_id)), None)
            .await
    }

    pub async fn remove_classes_from_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let teacher = self.find_one(Some(teacher_id), None).await?;
        let teacher_id = Self::id_to_string(teacher_id)?;
        let school_id = teacher
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let ids = teacher
            .class_ids
            .unwrap_or_default()
            .into_iter()
            .filter(|id| !class_ids.contains(id))
            .collect::<Vec<_>>();
        self.sync_id_relation("teacher_classes", "class_id", &teacher_id, &school_id, &ids)
            .await?;
        self.find_one(Some(&IdType::from_string(teacher_id)), None)
            .await
    }

    pub async fn remove_subjects_from_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let teacher = self.find_one(Some(teacher_id), None).await?;
        let teacher_id = Self::id_to_string(teacher_id)?;
        let school_id = teacher
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let ids = teacher
            .subject_ids
            .unwrap_or_default()
            .into_iter()
            .filter(|id| !subject_ids.contains(id))
            .collect::<Vec<_>>();
        self.sync_id_relation(
            "teacher_subjects",
            "class_subject_id",
            &teacher_id,
            &school_id,
            &ids,
        )
        .await?;
        self.find_one(Some(&IdType::from_string(teacher_id)), None)
            .await
    }
}
