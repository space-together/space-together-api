use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        announcement::AnnouncementWithRelations,
        common_details::{Gender, Paginated},
        parent::{
            AttendanceRecord, AttendanceSummary, ChildSummary, FinanceSummary, Parent,
            ParentDashboard, ParentPartial, ParentStatus, ParentWithRelations, PaymentRecord,
            StudentResults,
        },
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
pub struct ParentQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl ParentQuery {
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

pub struct ParentService {
    pub pool: PgPool,
}

impl ParentService {
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

    fn status_to_string(status: &ParentStatus) -> String {
        match status {
            ParentStatus::Active => "Active",
            ParentStatus::Inactive => "Inactive",
        }
        .to_string()
    }

    fn status_from_string(value: Option<String>) -> ParentStatus {
        match value
            .as_deref()
            .unwrap_or("Active")
            .to_ascii_lowercase()
            .as_str()
        {
            "inactive" => ParentStatus::Inactive,
            _ => ParentStatus::Active,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT
          id, school_id, user_id, name, email, phone, gender, image, image_id,
          relationship, occupation, national_id, status, is_active, created_at, updated_at
        FROM parents
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a ParentQuery>,
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
                "student_id" | "student_ids" => {
                    parse_object_id_value(value)?;
                    sql.push(
                        " AND EXISTS (
                          SELECT 1
                          FROM parent_student_links psl
                          LEFT JOIN student_school_enrollments sse ON sse.student_id = psl.student_id
                          WHERE psl.parent_id = parents.id
                            AND psl.status = 'active'
                            AND (psl.student_id = ",
                    )
                    .push_bind(value)
                    .push(" OR sse.id = ")
                    .push_bind(value)
                    .push("))");
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
                "relationship" => {
                    sql.push(" AND lower(coalesce(relationship, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "status" => {
                    sql.push(" AND lower(status) = lower(")
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
                        message: format!("Unsupported parent filter field: {}", field),
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
            .push(" OR lower(coalesce(relationship, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(status) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(user_id, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(school_id, '')) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    async fn enrollment_to_profile_id(&self, student_id: &str) -> Result<String, AppError> {
        parse_object_id_value(student_id)?;
        let profile_id = sqlx::query_scalar::<_, String>(
            "SELECT student_id FROM student_school_enrollments WHERE id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(profile_id.unwrap_or_else(|| student_id.to_string()))
    }

    async fn fetch_student_ids(
        &self,
        parent_id: &str,
        school_id: Option<&str>,
    ) -> Result<Option<Vec<ObjectId>>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COALESCE(sse.id, psl.student_id) AS student_id
            FROM parent_student_links psl
            LEFT JOIN student_school_enrollments sse
              ON sse.student_id = psl.student_id
             AND sse.deleted_at IS NULL
            WHERE psl.parent_id =
            "#,
        );
        query
            .push_bind(parent_id)
            .push(" AND psl.status = 'active'");
        if let Some(school_id) = school_id {
            query
                .push(" AND (psl.school_id = ")
                .push_bind(school_id)
                .push(" OR sse.school_id = ")
                .push_bind(school_id)
                .push(")");
        }
        query.push(" ORDER BY psl.created_at ASC");

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("student_id").map_err(Self::db_error)?;
                Self::parse_oid(&id, "student_id")
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn sync_student_links(&self, parent: &Parent) -> Result<(), AppError> {
        let Some(parent_id) = parent.id else {
            return Ok(());
        };
        let Some(student_ids) = &parent.student_ids else {
            return Ok(());
        };

        let parent_id = parent_id.to_hex();
        let school_id = parent.school_id.as_ref().map(|id| id.to_hex());

        sqlx::query("DELETE FROM parent_student_links WHERE parent_id = $1")
            .bind(&parent_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for student_id in student_ids {
            let profile_id = self.enrollment_to_profile_id(&student_id.to_hex()).await?;
            sqlx::query(
                r#"
                INSERT INTO parent_student_links (
                  id, parent_id, parent_user_id, student_id, school_id, relationship, status, starts_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, 'active', now())
                ON CONFLICT (parent_id, student_id, school_id)
                DO UPDATE SET status = 'active', relationship = EXCLUDED.relationship, updated_at = now()
                "#,
            )
            .bind(Self::new_id())
            .bind(&parent_id)
            .bind(parent.user_id.as_ref().map(|id| id.to_hex()))
            .bind(profile_id)
            .bind(&school_id)
            .bind(&parent.relationship)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn parent_from_row(&self, row: PgRow) -> Result<Parent, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let school_id: Option<String> = row.try_get("school_id").ok().flatten();
        let student_ids = self.fetch_student_ids(&id, school_id.as_deref()).await?;

        Ok(Parent {
            id: Some(Self::parse_oid(&id, "id")?),
            user_id: Self::parse_oid_opt(row.try_get("user_id").ok().flatten(), "user_id")?,
            school_id: Self::parse_oid_opt(school_id, "school_id")?,
            student_ids,
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
            relationship: row.try_get("relationship").ok().flatten(),
            occupation: row.try_get("occupation").ok().flatten(),
            national_id: row.try_get("national_id").ok().flatten(),
            status: Self::status_from_string(row.try_get("status").ok().flatten()),
            is_active: row.try_get("is_active").ok().flatten().unwrap_or(true),
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    async fn email_exists_in_school(
        &self,
        email: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<Parent>, AppError> {
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
            Some(row) => Ok(Some(self.parent_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(&self, dto: Parent) -> Result<Parent, AppError> {
        self.ensure_indexes().await?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;
        is_valid_email(&dto.email).map_err(|e| AppError { message: e })?;

        let school_id = dto.school_id.as_ref().map(|id| id.to_hex());
        if let Some(parent) = self
            .email_exists_in_school(&dto.email, school_id.as_deref(), None)
            .await?
        {
            return Err(AppError {
                message: format!("Email already exists: {}", parent.email),
            });
        }

        let mut parent = dto;
        if let Some(image_data) = parent.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            parent.image_id = Some(cloud_res.public_id);
            parent.image = Some(cloud_res.secure_url);
        }

        let id = parent
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO parents (
              id, school_id, user_id, name, email, phone, gender, image, image_id,
              relationship, occupation, national_id, status, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(parent.user_id.as_ref().map(|id| id.to_hex()))
        .bind(&parent.name)
        .bind(&parent.email)
        .bind(&parent.phone)
        .bind(parent.gender.as_ref().and_then(Self::enum_to_string))
        .bind(&parent.image)
        .bind(&parent.image_id)
        .bind(&parent.relationship)
        .bind(&parent.occupation)
        .bind(&parent.national_id)
        .bind(Self::status_to_string(&parent.status))
        .bind(parent.is_active)
        .bind(parent.created_at)
        .bind(parent.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let created = self.find_one(Some(&IdType::from_string(&id)), None).await?;
        self.sync_student_links(&created).await?;
        self.sync_parent_membership(&created).await?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    async fn sync_parent_membership(&self, parent: &Parent) -> Result<(), AppError> {
        let (Some(parent_id), Some(user_id), Some(school_id)) =
            (parent.id, parent.user_id, parent.school_id)
        else {
            return Ok(());
        };
        let parent_id = parent_id.to_hex();
        let user_id = user_id.to_hex();
        let school_id = school_id.to_hex();

        sqlx::query(
            r#"
            INSERT INTO school_memberships (id, school_id, user_id, school_user_id, member_type, status, joined_at)
            VALUES ($1, $2, $3, $4, 'PARENT', 'active', now())
            ON CONFLICT (school_id, user_id, member_type)
            DO UPDATE SET school_user_id = EXCLUDED.school_user_id, status = 'active', ended_at = NULL, updated_at = now()
            "#,
        )
        .bind(Self::new_id())
        .bind(&school_id)
        .bind(&user_id)
        .bind(&parent_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (id, user_id, school_id, role, scope, starts_at)
            VALUES ($1, $2, $3, 'PARENT', 'school', now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(Self::new_id())
        .bind(user_id)
        .bind(school_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<ParentQuery>,
    ) -> Result<Parent, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            let id = Self::id_to_string(id)?;
            sql.push(" AND id = ").push_bind(id);
        }
        sql.push(" ORDER BY updated_at DESC LIMIT 1");

        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(self.parent_from_row(row).await?),
            None => Err(AppError {
                message: "Parent not found".into(),
            }),
        }
    }

    pub async fn find_by_user_id(
        &self,
        user_id: &str,
        school_id: Option<&str>,
    ) -> Result<Parent, AppError> {
        self.find_one(
            None,
            Some(ParentQuery {
                school_id: school_id.map(ToOwned::to_owned),
                field_values: vec![("user_id".to_string(), user_id.to_string())],
                ..ParentQuery::default()
            }),
        )
        .await
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ParentQuery>,
    ) -> Result<Paginated<Parent>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM parents WHERE deleted_at IS NULL");
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
            data.push(self.parent_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &ParentPartial) -> Result<Parent, AppError> {
        if let Some(ref name) = update.name {
            is_valid_name(name).map_err(|e| AppError { message: e })?;
        }
        if let Some(ref email) = update.email {
            is_valid_email(email).map_err(|e| AppError { message: e })?;
        }

        let existing_parent = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;

        if let Some(email) = &update.email {
            if existing_parent.email != *email
                && self
                    .email_exists_in_school(
                        email,
                        existing_parent
                            .school_id
                            .as_ref()
                            .map(|id| id.to_hex())
                            .as_deref(),
                        Some(&id_string),
                    )
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("Email already exists: {}", email),
                });
            }
        }

        let mut update_data = update.clone();
        if let Some(Some(new_image_data)) = update.image.clone() {
            if Some(new_image_data.clone()) != existing_parent.image {
                if let Some(old_image_id) = existing_parent.image_id.clone() {
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

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE parents SET updated_at = now()");
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
        set_optional!(relationship, "relationship");
        set_optional!(occupation, "occupation");
        set_optional!(national_id, "national_id");

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
        if let Some(value) = update_data.gender {
            sql.push(", gender = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched = true;
        }
        if let Some(value) = update_data.status {
            sql.push(", status = ")
                .push_bind(Self::status_to_string(&value));
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

        if let Some(student_ids) = &update_data.student_ids {
            let mut parent = self.find_one(Some(id), None).await?;
            parent.student_ids = student_ids.clone();
            self.sync_student_links(&parent).await?;
        }

        if !touched && update_data.student_ids.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let parent = self.find_one(Some(id), None).await?;
        self.sync_parent_membership(&parent).await?;
        Ok(parent)
    }

    pub async fn delete(&self, id: &IdType) -> Result<Parent, AppError> {
        let parent = self.find_one(Some(id), None).await?;
        if let Some(ref image_id) = parent.image_id {
            CloudinaryService::delete_from_cloudinary(image_id)
                .await
                .ok();
        }
        let id = Self::id_to_string(id)?;
        sqlx::query("UPDATE parents SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(parent)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ParentQuery>,
    ) -> Result<Paginated<ParentWithRelations>, AppError> {
        let parents = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(parents.data.len());
        for parent in parents.data {
            data.push(self.with_relations(parent).await?);
        }

        Ok(Paginated {
            data,
            total: parents.total,
            total_pages: parents.total_pages,
            current_page: parents.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<ParentQuery>,
    ) -> Result<ParentWithRelations, AppError> {
        let parent = self.find_one(id, query).await?;
        self.with_relations(parent).await
    }

    async fn with_relations(&self, parent: Parent) -> Result<ParentWithRelations, AppError> {
        let user_repo = UserRepo::new(&self.pool);
        let school_service = SchoolService::new(&self.pool);

        let user = match parent.user_id {
            Some(user_id) => {
                user_repo
                    .find_by_id(&IdType::from_object_id(user_id))
                    .await?
            }
            None => None,
        };
        let school = match parent.school_id {
            Some(school_id) => school_service
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };

        let mut students = Vec::new();
        if let Some(student_ids) = &parent.student_ids {
            let student_service = crate::services::student_service::StudentService::new(&self.pool);
            for student_id in student_ids {
                if let Ok(student) = student_service
                    .find_one(Some(&IdType::from_object_id(*student_id)), None)
                    .await
                {
                    students.push(student);
                }
            }
        }

        Ok(ParentWithRelations {
            parent,
            user,
            school,
            students: (!students.is_empty()).then_some(students),
        })
    }

    pub async fn count_parents(
        &self,
        filter: Option<String>,
        query: Option<ParentQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub async fn validate_parent_student_access(
        &self,
        parent_id: &IdType,
        student_id: &IdType,
    ) -> Result<bool, AppError> {
        let parent_id = Self::id_to_string(parent_id)?;
        let student_id = Self::id_to_string(student_id)?;
        let profile_id = self.enrollment_to_profile_id(&student_id).await?;

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)
            FROM parent_student_links
            WHERE parent_id = $1
              AND student_id = $2
              AND status = 'active'
              AND (ends_at IS NULL OR ends_at > now())
            "#,
        )
        .bind(parent_id)
        .bind(profile_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(count > 0)
    }

    pub async fn get_dashboard(
        &self,
        parent_id: &IdType,
        school_id: &str,
        state: &AppState,
    ) -> Result<ParentDashboard, AppError> {
        let parent = self.find_one(Some(parent_id), None).await?;
        let student_ids = parent.student_ids.clone().unwrap_or_default();
        let total_children = student_ids.len() as i64;

        let mut children_summary = Vec::new();
        for student_id in student_ids {
            if let Ok(summary) = self
                .get_child_summary(&student_id.to_hex(), school_id, state)
                .await
            {
                children_summary.push(summary);
            }
        }

        Ok(ParentDashboard {
            total_children,
            latest_announcements: Vec::<AnnouncementWithRelations>::new(),
            children_summary,
        })
    }

    async fn get_child_summary(
        &self,
        student_id: &str,
        _school_id: &str,
        _state: &AppState,
    ) -> Result<ChildSummary, AppError> {
        let student_service = crate::services::student_service::StudentService::new(&self.pool);
        let student = student_service
            .find_one(Some(&IdType::from_string(student_id)), None)
            .await?;

        Ok(ChildSummary {
            student_id: student.id,
            student_name: student.name,
            class_name: None,
            attendance_percentage: 85.0,
            current_term_gpa: 3.5,
            outstanding_fees: 0.0,
        })
    }

    pub async fn get_attendance_summary(
        &self,
        student_id: &str,
        school_id: &str,
        _state: &AppState,
    ) -> Result<AttendanceSummary, AppError> {
        let profile_id = self.enrollment_to_profile_id(student_id).await?;
        let rows = sqlx::query(
            r#"
            SELECT status, count(*)::bigint AS count
            FROM attendance
            WHERE school_id = $1 AND student_id = $2 AND deleted_at IS NULL
            GROUP BY status
            "#,
        )
        .bind(school_id)
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut present_count = 0;
        let mut absent_count = 0;
        let mut late_count = 0;
        let mut excused_count = 0;

        for row in rows {
            let status: String = row.try_get("status").map_err(Self::db_error)?;
            let count: i64 = row.try_get("count").map_err(Self::db_error)?;
            match status.as_str() {
                "Present" | "present" => present_count = count,
                "Absent" | "absent" => absent_count = count,
                "Late" | "late" => late_count = count,
                "Excused" | "excused" => excused_count = count,
                _ => {}
            }
        }

        let recent_rows = sqlx::query(
            r#"
            SELECT date::text AS date, status, note
            FROM attendance
            WHERE school_id = $1 AND student_id = $2 AND deleted_at IS NULL
            ORDER BY date DESC
            LIMIT 10
            "#,
        )
        .bind(school_id)
        .bind(self.enrollment_to_profile_id(student_id).await?)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut recent_records = Vec::new();
        for row in recent_rows {
            let date_text: String = row.try_get("date").map_err(Self::db_error)?;
            let date = chrono::NaiveDate::parse_from_str(&date_text, "%Y-%m-%d")
                .ok()
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .map(|naive| chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
                .unwrap_or_else(Utc::now);
            recent_records.push(AttendanceRecord {
                date,
                status: row.try_get("status").map_err(Self::db_error)?,
                remarks: row.try_get("note").ok().flatten(),
            });
        }

        let total_days = present_count + absent_count + late_count + excused_count;
        let attendance_percentage = if total_days > 0 {
            (present_count as f64 / total_days as f64) * 100.0
        } else {
            0.0
        };

        Ok(AttendanceSummary {
            present_count,
            absent_count,
            late_count,
            excused_count,
            total_days,
            attendance_percentage,
            recent_records,
        })
    }

    pub async fn get_student_results(
        &self,
        student_id: &str,
        school_id: &str,
        education_year_id: Option<&str>,
        term_id: Option<&str>,
        _state: &AppState,
    ) -> Result<StudentResults, AppError> {
        let profile_id = self.enrollment_to_profile_id(student_id).await?;
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT average_score, rank, status
            FROM student_term_results
            WHERE school_id =
            "#,
        );
        query
            .push_bind(school_id)
            .push(" AND student_id = ")
            .push_bind(profile_id)
            .push(" AND deleted_at IS NULL");
        if let Some(year_id) = education_year_id {
            query.push(" AND academic_year = ").push_bind(year_id);
        }
        if let Some(term_id) = term_id {
            query.push(" AND term = ").push_bind(term_id);
        }
        query.push(" ORDER BY updated_at DESC LIMIT 1");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .ok_or_else(|| AppError {
                message: "No results found for this student".into(),
            })?;

        let average_score: Option<f64> = row.try_get("average_score").ok().flatten();
        Ok(StudentResults {
            term_gpa: average_score.unwrap_or(0.0),
            rank: row.try_get("rank").ok().flatten(),
            total_students: None,
            grade: row
                .try_get::<Option<String>, _>("status")
                .ok()
                .flatten()
                .unwrap_or_else(|| "Pending".to_string()),
            subject_results: vec![],
            teacher_remarks: None,
        })
    }

    pub async fn get_finance_summary(
        &self,
        student_id: &str,
        school_id: &str,
        _state: &AppState,
    ) -> Result<FinanceSummary, AppError> {
        let profile_id = self.enrollment_to_profile_id(student_id).await?;
        let row = sqlx::query(
            r#"
            SELECT
              COALESCE(sum(amount) FILTER (WHERE status <> 'paid'), 0)::float8 AS outstanding,
              COALESCE(sum(amount) FILTER (WHERE status = 'paid'), 0)::float8 AS paid,
              COALESCE(sum(amount), 0)::float8 AS total
            FROM finance_records
            WHERE school_id = $1 AND student_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(school_id)
        .bind(profile_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(FinanceSummary {
            total_fee_required: row.try_get("total").ok().unwrap_or(0.0),
            amount_paid: row.try_get("paid").ok().unwrap_or(0.0),
            outstanding_balance: row.try_get("outstanding").ok().unwrap_or(0.0),
            payment_history: Vec::<PaymentRecord>::new(),
            installments: vec![],
        })
    }

    pub async fn get_parent_announcements(
        &self,
        parent_id: &IdType,
        _school_id: &str,
        limit: Option<i64>,
        skip: Option<i64>,
        _state: &AppState,
    ) -> Result<Paginated<AnnouncementWithRelations>, AppError> {
        self.find_one(Some(parent_id), None).await?;
        Ok(Paginated {
            data: Vec::<AnnouncementWithRelations>::new(),
            total: 0,
            total_pages: 0,
            current_page: (skip.unwrap_or(0) / limit.unwrap_or(20).max(1)) + 1,
        })
    }

    pub async fn is_parent_of(
        &self,
        parent_id: &IdType,
        student_id: &IdType,
    ) -> Result<bool, AppError> {
        self.validate_parent_student_access(parent_id, student_id)
            .await
    }
}
