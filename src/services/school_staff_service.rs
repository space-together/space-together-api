use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        school_staff::{SchoolStaff, SchoolStaffPartial, SchoolStaffType},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    services::cloudinary_service::CloudinaryService,
    utils::{
        email::is_valid_email,
        names::is_valid_name,
        object_id::{parse_object_id_value, ObjectId},
    },
};

#[derive(Debug, Clone, Default)]
pub struct SchoolStaffQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl SchoolStaffQuery {
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

pub struct SchoolStaffService {
    pub pool: PgPool,
}

impl SchoolStaffService {
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

    fn staff_type_to_string(value: &SchoolStaffType) -> String {
        match value {
            SchoolStaffType::Director => "Director",
            SchoolStaffType::HeadOfStudies => "HeadOfStudies",
        }
        .to_string()
    }

    fn staff_type_from_string(value: Option<String>) -> SchoolStaffType {
        match value
            .as_deref()
            .unwrap_or("HeadOfStudies")
            .to_ascii_lowercase()
            .as_str()
        {
            "director" => SchoolStaffType::Director,
            _ => SchoolStaffType::HeadOfStudies,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, user_id, creator_id, name, email, staff_type, is_active,
               image, image_id, created_at, updated_at
        FROM school_staff
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a SchoolStaffQuery>,
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
                "type" => {
                    sql.push(" AND lower(staff_type) = lower(")
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
                        message: format!("Unsupported school staff filter field: {}", field),
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
            .push(" OR lower(staff_type) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(" OR EXISTS (SELECT 1 FROM school_staff_tags sst WHERE sst.staff_id = school_staff.id AND lower(sst.tag) LIKE ")
            .push_bind(search_like)
            .push("))");
    }

    async fn fetch_tags(&self, staff_id: &str) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT tag
            FROM school_staff_tags
            WHERE staff_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(staff_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| row.try_get("tag").map_err(Self::db_error))
            .collect()
    }

    async fn sync_tags(&self, staff_id: &str, tags: &[String]) -> Result<(), AppError> {
        sqlx::query("DELETE FROM school_staff_tags WHERE staff_id = $1")
            .bind(staff_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, tag) in tags.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO school_staff_tags (id, staff_id, tag, position)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(Self::new_id())
            .bind(staff_id)
            .bind(tag)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn staff_from_row(&self, row: PgRow) -> Result<SchoolStaff, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(SchoolStaff {
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
            r#type: Self::staff_type_from_string(row.try_get("staff_type").ok().flatten()),
            is_active: row.try_get("is_active").ok().flatten().unwrap_or(true),
            tags: self.fetch_tags(&id).await?,
            image: row.try_get("image").ok().flatten(),
            image_id: row.try_get("image_id").ok().flatten(),
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    async fn email_exists(
        &self,
        email: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<SchoolStaff>, AppError> {
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
            Some(row) => Ok(Some(self.staff_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(
        &self,
        mut dto: SchoolStaff,
        _state: Option<&AppState>,
    ) -> Result<SchoolStaff, AppError> {
        self.ensure_indexes().await?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;
        is_valid_email(&dto.email).map_err(|e| AppError { message: e })?;

        let school_id = dto.school_id.as_ref().map(|id| id.to_hex());
        if let Some(existing) = self
            .email_exists(&dto.email, school_id.as_deref(), None)
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
            INSERT INTO school_staff (
              id, school_id, user_id, creator_id, name, email, staff_type,
              is_active, image, image_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(dto.user_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.creator_id.as_ref().map(|id| id.to_hex()))
        .bind(&dto.name)
        .bind(&dto.email)
        .bind(Self::staff_type_to_string(&dto.r#type))
        .bind(dto.is_active)
        .bind(&dto.image)
        .bind(&dto.image_id)
        .bind(dto.created_at)
        .bind(dto.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_tags(&id, &dto.tags).await?;
        let staff = self.find_one(Some(&IdType::from_string(&id)), None).await?;
        self.sync_membership(&staff).await?;
        Ok(staff)
    }

    async fn sync_membership(&self, staff: &SchoolStaff) -> Result<(), AppError> {
        let (Some(staff_id), Some(user_id), Some(school_id)) =
            (staff.id, staff.user_id, staff.school_id)
        else {
            return Ok(());
        };

        sqlx::query(
            r#"
            INSERT INTO school_memberships (id, school_id, user_id, school_user_id, member_type, status, joined_at)
            VALUES ($1, $2, $3, $4, 'SCHOOLSTAFF', 'active', now())
            ON CONFLICT (school_id, user_id, member_type)
            DO UPDATE SET school_user_id = EXCLUDED.school_user_id, status = 'active', ended_at = NULL, updated_at = now()
            "#,
        )
        .bind(Self::new_id())
        .bind(school_id.to_hex())
        .bind(user_id.to_hex())
        .bind(staff_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (id, user_id, school_id, role, scope, starts_at)
            VALUES ($1, $2, $3, 'SCHOOLSTAFF', 'school', now())
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
        query: Option<SchoolStaffQuery>,
    ) -> Result<SchoolStaff, AppError> {
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
            Some(row) => Ok(self.staff_from_row(row).await?),
            None => Err(AppError {
                message: "School staff not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<SchoolStaffQuery>,
    ) -> Result<Paginated<SchoolStaff>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM school_staff WHERE deleted_at IS NULL",
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
            data.push(self.staff_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &SchoolStaffPartial,
    ) -> Result<SchoolStaff, AppError> {
        if let Some(ref name) = update.name {
            is_valid_name(name).map_err(|e| AppError { message: e })?;
        }
        if let Some(ref email) = update.email {
            is_valid_email(email).map_err(|e| AppError { message: e })?;
        }

        let existing = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;

        if let Some(ref email) = update.email {
            if existing.email != *email
                && self
                    .email_exists(
                        email,
                        existing.school_id.as_ref().map(|id| id.to_hex()).as_deref(),
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
            if Some(new_image_data.clone()) != existing.image {
                if let Some(old_image_id) = existing.image_id.clone() {
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

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE school_staff SET updated_at = now()");
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
        if let Some(value) = update_data.r#type {
            sql.push(", staff_type = ")
                .push_bind(Self::staff_type_to_string(&value));
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

        if let Some(tags) = &update_data.tags {
            self.sync_tags(&id_string, tags).await?;
        }

        if !touched && update_data.tags.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let staff = self.find_one(Some(id), None).await?;
        self.sync_membership(&staff).await?;
        Ok(staff)
    }

    pub async fn delete(&self, id: &IdType) -> Result<SchoolStaff, AppError> {
        let staff = self.find_one(Some(id), None).await?;
        let id = Self::id_to_string(id)?;
        sqlx::query("UPDATE school_staff SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(staff)
    }

    pub async fn count_staff(
        &self,
        filter: Option<String>,
        query: Option<SchoolStaffQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }
}
