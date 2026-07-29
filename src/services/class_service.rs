use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        class::{Class, ClassLevelType, ClassSettings, ClassType, ClassWithOthers, UpdateClass},
        common_details::{Image, Paginated},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    repositories::user_repo::UserRepo,
    services::{
        cloudinary_service::CloudinaryService, school_service::SchoolService,
        teacher_service::TeacherService,
    },
    utils::{
        code::generate_code,
        names::is_valid_username,
        object_id::{parse_object_id_value, ObjectId},
    },
};

#[derive(Debug, Clone, Default)]
pub struct ClassQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl ClassQuery {
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

pub struct ClassService {
    pub pool: PgPool,
}

impl ClassService {
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

    fn class_type_from_string(value: Option<String>) -> ClassType {
        Self::enum_from_string(value).unwrap_or_default()
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, name, username, code, school_id, creator_id, class_teacher_id,
               class_type, level_type, parent_class_id, main_class_id, trade_id,
               is_active, image_id, image, description, capacity, subject, grade_level,
               created_at, updated_at
        FROM classes
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a ClassQuery>,
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
                "school_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND school_id = ").push_bind(value);
                }
                "creator_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND creator_id = ").push_bind(value);
                }
                "class_teacher_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND class_teacher_id = ").push_bind(value);
                }
                "parent_class_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND parent_class_id = ").push_bind(value);
                }
                "main_class_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND main_class_id = ").push_bind(value);
                }
                "trade_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND trade_id = ").push_bind(value);
                }
                "username" => {
                    sql.push(" AND lower(coalesce(username, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "code" => {
                    sql.push(" AND lower(coalesce(code, '')) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "name" => {
                    sql.push(" AND lower(name) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "type" => {
                    sql.push(" AND lower(class_type) = lower(")
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
                        message: format!("Unsupported class filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(username, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(code, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(class_type, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(" OR EXISTS (SELECT 1 FROM class_tags ct WHERE ct.class_id = classes.id AND lower(ct.tag) LIKE ")
            .push_bind(search_like)
            .push("))");
    }

    async fn fetch_tags(&self, class_id: &str) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query("SELECT tag FROM class_tags WHERE class_id = $1 ORDER BY position")
            .bind(class_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| row.try_get("tag").map_err(Self::db_error))
            .collect()
    }

    async fn sync_tags(&self, class_id: &str, tags: &[String]) -> Result<(), AppError> {
        sqlx::query("DELETE FROM class_tags WHERE class_id = $1")
            .bind(class_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, tag) in tags.iter().enumerate() {
            sqlx::query(
                "INSERT INTO class_tags (id, class_id, tag, position) VALUES ($1, $2, $3, $4)",
            )
            .bind(Self::new_id())
            .bind(class_id)
            .bind(tag)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }
        Ok(())
    }

    async fn fetch_background_images(
        &self,
        class_id: &str,
    ) -> Result<Option<Vec<Image>>, AppError> {
        let rows = sqlx::query(
            "SELECT public_id, url FROM class_background_images WHERE class_id = $1 ORDER BY position",
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let images = rows
            .into_iter()
            .map(|row| {
                Ok(Image {
                    id: row.try_get("public_id").map_err(Self::db_error)?,
                    url: row.try_get("url").map_err(Self::db_error)?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((!images.is_empty()).then_some(images))
    }

    async fn sync_background_images(
        &self,
        class_id: &str,
        images: &[Image],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM class_background_images WHERE class_id = $1")
            .bind(class_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, image) in images.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO class_background_images (id, class_id, public_id, url, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(class_id)
            .bind(&image.id)
            .bind(&image.url)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }
        Ok(())
    }

    async fn fetch_subclass_ids(&self, class_id: &str) -> Result<Option<Vec<ObjectId>>, AppError> {
        let rows = sqlx::query(
            "SELECT id FROM classes WHERE parent_class_id = $1 AND deleted_at IS NULL ORDER BY username, name",
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("id").map_err(Self::db_error)?;
                Self::parse_oid(&id, "subclass_id")
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn class_from_row(&self, row: PgRow) -> Result<Class, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Class {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            username: row.try_get("username").map_err(Self::db_error)?,
            code: row.try_get("code").ok().flatten(),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            creator_id: Self::parse_oid_opt(
                row.try_get("creator_id").ok().flatten(),
                "creator_id",
            )?,
            class_teacher_id: Self::parse_oid_opt(
                row.try_get("class_teacher_id").ok().flatten(),
                "class_teacher_id",
            )?,
            r#type: Self::class_type_from_string(row.try_get("class_type").ok().flatten()),
            level_type: Self::enum_from_string::<ClassLevelType>(
                row.try_get("level_type").ok().flatten(),
            ),
            parent_class_id: Self::parse_oid_opt(
                row.try_get("parent_class_id").ok().flatten(),
                "parent_class_id",
            )?,
            subclass_ids: self.fetch_subclass_ids(&id).await?,
            main_class_id: Self::parse_oid_opt(
                row.try_get("main_class_id").ok().flatten(),
                "main_class_id",
            )?,
            trade_id: Self::parse_oid_opt(row.try_get("trade_id").ok().flatten(), "trade_id")?,
            is_active: row.try_get("is_active").ok().flatten(),
            image_id: row.try_get("image_id").ok().flatten(),
            image: row.try_get("image").ok().flatten(),
            background_images: self.fetch_background_images(&id).await?,
            description: row.try_get("description").ok().flatten(),
            capacity: row
                .try_get::<Option<i32>, _>("capacity")
                .ok()
                .flatten()
                .and_then(|value| u32::try_from(value).ok()),
            subject: row.try_get("subject").ok().flatten(),
            grade_level: row.try_get("grade_level").ok().flatten(),
            tags: self.fetch_tags(&id).await?,
            settings: None,
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    async fn username_exists(
        &self,
        username: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<Class>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(Self::select_sql());
        query
            .push(" AND lower(username) = lower(")
            .push_bind(username)
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
            Some(row) => Ok(Some(self.class_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(&self, mut dto: Class) -> Result<Class, AppError> {
        self.ensure_indexes().await?;
        is_valid_username(&dto.username).map_err(|e| AppError { message: e })?;

        let school_id = dto
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;

        if self
            .username_exists(&dto.username, Some(&school_id), None)
            .await?
            .is_some()
        {
            return Err(AppError {
                message: format!("Class username already exists: {}", dto.username),
            });
        }

        if let Some(image_data) = dto.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            dto.image_id = Some(cloud_res.public_id);
            dto.image = Some(cloud_res.secure_url);
        }

        if let Some(background_images_data) = dto.background_images.clone() {
            let mut uploaded_images = Vec::new();
            for bg in background_images_data {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url)
                    .await
                    .map_err(|e| AppError { message: e })?;
                uploaded_images.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            dto.background_images = Some(uploaded_images);
        }

        let id = dto
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let now = Utc::now();
        let code = dto.code.clone().unwrap_or_else(generate_code);
        let capacity = dto.capacity.and_then(|value| i32::try_from(value).ok());

        sqlx::query(
            r#"
            INSERT INTO classes (
              id, school_id, creator_id, class_teacher_id, parent_class_id, main_class_id, trade_id,
              name, username, code, class_type, level_type, is_active, image_id, image,
              description, capacity, subject, grade_level, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(dto.creator_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.class_teacher_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.parent_class_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.main_class_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.trade_id.as_ref().map(|id| id.to_hex()))
        .bind(&dto.name)
        .bind(&dto.username)
        .bind(&code)
        .bind(Self::enum_to_string(&dto.r#type).unwrap_or_else(|| "Private".into()))
        .bind(dto.level_type.as_ref().and_then(Self::enum_to_string))
        .bind(dto.is_active.unwrap_or(true))
        .bind(&dto.image_id)
        .bind(&dto.image)
        .bind(&dto.description)
        .bind(capacity)
        .bind(&dto.subject)
        .bind(&dto.grade_level)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_tags(&id, &dto.tags).await?;
        if let Some(images) = &dto.background_images {
            self.sync_background_images(&id, images).await?;
        }

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<ClassQuery>,
    ) -> Result<Class, AppError> {
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
            Some(row) => self.class_from_row(row).await,
            None => Err(AppError {
                message: "Class not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ClassQuery>,
    ) -> Result<Paginated<Class>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM classes WHERE deleted_at IS NULL");
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
            data.push(self.class_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &UpdateClass) -> Result<Class, AppError> {
        if let Some(ref username) = update.username {
            is_valid_username(username).map_err(|e| AppError { message: e })?;
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

        if let Some(ref username) = update.username {
            if existing.username != *username
                && self
                    .username_exists(username, Some(&school_id), Some(&id_string))
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("Username already exists: {}", username),
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

        if let Some(Some(bg_images)) = update.background_images.clone() {
            if let Some(old_bgs) = existing.background_images.clone() {
                for bg in old_bgs {
                    let _ = CloudinaryService::delete_from_cloudinary(&bg.id).await;
                }
            }
            let mut uploaded_bgs = Vec::new();
            for bg in bg_images {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to upload background image: {}", e),
                    })?;
                uploaded_bgs.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            update_data.background_images = Some(Some(uploaded_bgs));
        }

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE classes SET updated_at = now()");
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
        set_required!(username, "username");
        set_optional!(code, "code");
        set_optional!(image_id, "image_id");
        set_optional!(image, "image");
        set_optional!(description, "description");
        set_optional!(subject, "subject");
        set_optional!(grade_level, "grade_level");

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
        if let Some(value) = update_data.class_teacher_id {
            sql.push(", class_teacher_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.parent_class_id {
            sql.push(", parent_class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.main_class_id {
            sql.push(", main_class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.trade_id {
            sql.push(", trade_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.r#type {
            sql.push(", class_type = ")
                .push_bind(Self::enum_to_string(&value).unwrap_or_else(|| "Private".into()));
            touched = true;
        }
        if let Some(value) = update_data.level_type {
            sql.push(", level_type = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched = true;
        }
        if let Some(value) = update_data.is_active {
            sql.push(", is_active = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update_data.capacity {
            sql.push(", capacity = ")
                .push_bind(value.and_then(|value| i32::try_from(value).ok()));
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
        if let Some(Some(images)) = &update_data.background_images {
            self.sync_background_images(&id_string, images).await?;
        }

        if !touched && update_data.tags.is_none() && update_data.background_images.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.find_one(Some(id), None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Class, AppError> {
        let class = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        sqlx::query("UPDATE classes SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(id_string)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(class)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ClassQuery>,
    ) -> Result<Paginated<ClassWithOthers>, AppError> {
        let classes = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(classes.data.len());
        for class in classes.data {
            data.push(self.with_relations(class).await?);
        }

        Ok(Paginated {
            data,
            total: classes.total,
            total_pages: classes.total_pages,
            current_page: classes.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<ClassQuery>,
    ) -> Result<ClassWithOthers, AppError> {
        let class = self.find_one(id, query).await?;
        self.with_relations(class).await
    }

    async fn with_relations(&self, class: Class) -> Result<ClassWithOthers, AppError> {
        let user_repo = UserRepo::new(&self.pool);
        let school_service = SchoolService::new(&self.pool);
        let teacher_service = TeacherService::new(&self.pool);

        let school = match class.school_id {
            Some(school_id) => school_service
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };
        let creator = match class.creator_id {
            Some(user_id) => {
                user_repo
                    .find_by_id(&IdType::from_object_id(user_id))
                    .await?
            }
            None => None,
        };
        let class_teacher = match class.class_teacher_id {
            Some(teacher_id) => teacher_service
                .find_one(Some(&IdType::from_object_id(teacher_id)), None)
                .await
                .ok(),
            None => None,
        };

        Ok(ClassWithOthers {
            class,
            school,
            creator,
            class_teacher,
            main_class: None,
            trade: None,
        })
    }

    pub async fn count_classes(
        &self,
        filter: Option<String>,
        query: Option<ClassQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub async fn create_many(&self, classes: Vec<Class>) -> Result<Vec<Class>, AppError> {
        let mut inserted = Vec::with_capacity(classes.len());
        for class in classes {
            inserted.push(self.create(class).await?);
        }
        Ok(inserted)
    }

    pub async fn create_many_sub_class_by_class_id(
        &self,
        main_class_id: &IdType,
        num_sub_classes: u8,
        creator_id: ObjectId,
    ) -> Result<Vec<Class>, AppError> {
        if !(1..=12).contains(&num_sub_classes) {
            return Err(AppError {
                message: "Number of sub-classes must be between 1 and 12".into(),
            });
        }

        let main_class = self.find_one(Some(main_class_id), None).await?;
        let main_obj_id = IdType::to_object_id(main_class_id)?;
        let start_index = main_class.subclass_ids.as_ref().map_or(0, |ids| ids.len()) as u8;
        let mut subclasses = Vec::new();

        for i in 0..num_sub_classes {
            let letter = ((b'A' + start_index + i) as char).to_string();
            let now = Utc::now();
            subclasses.push(Class {
                id: None,
                name: format!("{} {}", main_class.name, letter),
                username: format!("{}_{}", main_class.username, letter.to_lowercase()),
                code: Some(generate_code()),
                school_id: main_class.school_id,
                creator_id: Some(creator_id),
                class_teacher_id: None,
                r#type: main_class.r#type.clone(),
                level_type: Some(ClassLevelType::SubClass),
                parent_class_id: Some(main_obj_id),
                subclass_ids: None,
                main_class_id: main_class.main_class_id,
                trade_id: main_class.trade_id,
                is_active: Some(true),
                image_id: main_class.image_id.clone(),
                image: main_class.image.clone(),
                background_images: main_class.background_images.clone(),
                description: Some(format!("Sub class of {}", main_class.name)),
                capacity: main_class.capacity,
                subject: main_class.subject.clone(),
                grade_level: main_class.grade_level.clone(),
                tags: vec!["subclass".to_string()],
                settings: Some(ClassSettings::default()),
                created_at: now,
                updated_at: now,
            });
        }

        self.create_many(subclasses).await
    }
}
