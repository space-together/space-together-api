use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::Paginated,
        learning_material::{
            LearningMaterial, LearningMaterialPartial, LearningMaterialWithRelations, MaterialType,
        },
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::cloudinary_service::CloudinaryService,
    utils::object_id::ObjectId,
};

pub struct LearningMaterialService {
    pub pool: PgPool,
}

impl LearningMaterialService {
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

    fn material_type_to_string(value: &MaterialType) -> &'static str {
        match value {
            MaterialType::LessonNote => "LESSON_NOTE",
            MaterialType::Resource => "RESOURCE",
            MaterialType::Video => "VIDEO",
            MaterialType::File => "FILE",
        }
    }

    fn material_type_from_string(value: Option<String>) -> MaterialType {
        match value.unwrap_or_else(|| "FILE".to_string()).as_str() {
            "LESSON_NOTE" => MaterialType::LessonNote,
            "RESOURCE" => MaterialType::Resource,
            "VIDEO" => MaterialType::Video,
            _ => MaterialType::File,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, class_id, COALESCE(subject_id, class_subject_id) AS subject_id,
               uploaded_by, title, description, material_type,
               COALESCE(file_url, url) AS file_url, file_public_id, video_url,
               is_published, deleted_at, created_at, updated_at
        FROM learning_materials
        WHERE deleted_at IS NULL
        "#
    }

    fn material_from_row(row: &sqlx::postgres::PgRow) -> Result<LearningMaterial, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(LearningMaterial {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            subject_id: Self::parse_oid_opt(
                row.try_get("subject_id").ok().flatten(),
                "subject_id",
            )?,
            title: row.try_get("title").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            material_type: Self::material_type_from_string(
                row.try_get("material_type").ok().flatten(),
            ),
            file_url: row.try_get("file_url").ok().flatten(),
            file_public_id: row.try_get("file_public_id").ok().flatten(),
            video_url: row.try_get("video_url").ok().flatten(),
            uploaded_by: Self::parse_oid_opt(
                row.try_get("uploaded_by").ok().flatten(),
                "uploaded_by",
            )?,
            is_published: row.try_get("is_published").unwrap_or(false),
            deleted_at: row.try_get("deleted_at").ok().flatten(),
            created_at: row
                .try_get::<Option<DateTime<Utc>>, _>("created_at")
                .ok()
                .flatten()
                .unwrap_or_else(Utc::now),
            updated_at: row
                .try_get::<Option<DateTime<Utc>>, _>("updated_at")
                .ok()
                .flatten()
                .unwrap_or_else(Utc::now),
        })
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
        school_id: Option<&'a str>,
        only_published: bool,
    ) -> Result<(), AppError> {
        if only_published {
            sql.push(" AND is_published = true");
        }

        if let Some(school_id) = school_id {
            Self::parse_oid(school_id, "school_id")?;
            sql.push(" AND school_id = ").push_bind(school_id);
        }

        let Some(query) = query else {
            return Ok(());
        };

        if let Some(class_id) = query.class_id.as_deref() {
            Self::parse_oid(class_id, "class_id")?;
            sql.push(" AND class_id = ").push_bind(class_id);
        }

        if !query.by_ids.is_empty() {
            sql.push(" AND id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                Self::parse_oid(id, "id")?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }

        for (field, value) in query.field.iter().zip(query.value.iter()) {
            match field.as_str() {
                "_id" | "id" => {
                    Self::parse_oid(value, "id")?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "school_id" => {
                    Self::parse_oid(value, "school_id")?;
                    sql.push(" AND school_id = ").push_bind(value);
                }
                "class_id" => {
                    Self::parse_oid(value, "class_id")?;
                    sql.push(" AND class_id = ").push_bind(value);
                }
                "subject_id" | "class_subject_id" => {
                    Self::parse_oid(value, "subject_id")?;
                    sql.push(" AND COALESCE(subject_id, class_subject_id) = ")
                        .push_bind(value);
                }
                "uploaded_by" => {
                    Self::parse_oid(value, "uploaded_by")?;
                    sql.push(" AND uploaded_by = ").push_bind(value);
                }
                "material_type" => {
                    sql.push(" AND lower(material_type) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                "is_published" => {
                    let published = value.eq_ignore_ascii_case("true");
                    sql.push(" AND is_published = ").push_bind(published);
                }
                "title" => {
                    sql.push(" AND lower(title) LIKE ")
                        .push_bind(format!("%{}%", value.to_ascii_lowercase()));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, filter: &'a str) {
        let search = format!("%{}%", filter.to_ascii_lowercase());
        sql.push(" AND (lower(title) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(description, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(material_type, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(id) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(class_id, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(subject_id, class_subject_id, '')) LIKE ")
            .push_bind(search)
            .push(")");
    }

    pub async fn create(
        &self,
        dto: LearningMaterial,
        file_bytes: Option<Vec<u8>>,
        file_name: Option<String>,
        _user: &AuthUserDto,
        _state: &AppState,
    ) -> Result<LearningMaterial, AppError> {
        self.ensure_indexes().await?;

        if dto.title.trim().is_empty() {
            return Err(AppError {
                message: "Title is required".into(),
            });
        }

        let mut material = dto;

        if let (Some(bytes), Some(name)) = (file_bytes, file_name) {
            let school_id = material.school_id.ok_or(AppError {
                message: "School ID is required".into(),
            })?;
            let subject_id = material.subject_id.ok_or(AppError {
                message: "Subject ID is required".into(),
            })?;
            let folder = format!(
                "space-together/{}/subjects/{}",
                school_id.to_hex(),
                subject_id.to_hex()
            );

            let upload_result = CloudinaryService::upload_file(bytes, &name, &folder)
                .await
                .map_err(|e| AppError { message: e })?;
            material.file_url = Some(upload_result.url);
            material.file_public_id = Some(upload_result.public_id);
        }

        if matches!(material.material_type, MaterialType::Video) && material.video_url.is_none() {
            return Err(AppError {
                message: "Video URL is required for VIDEO type".into(),
            });
        }

        let id = material
            .id
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let school_id = material.school_id.ok_or(AppError {
            message: "School ID is required".into(),
        })?;
        let material_type = Self::material_type_to_string(&material.material_type);

        sqlx::query(
            r#"
            INSERT INTO learning_materials (
              id, school_id, class_id, class_subject_id, subject_id, uploaded_by,
              title, description, material_type, url, file_url, file_public_id,
              video_url, is_published, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $4, $5, $6, $7, $8, $9, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&id)
        .bind(school_id.to_hex())
        .bind(material.class_id.map(|id| id.to_hex()))
        .bind(material.subject_id.map(|id| id.to_hex()))
        .bind(material.uploaded_by.map(|id| id.to_hex()))
        .bind(material.title)
        .bind(material.description)
        .bind(material_type)
        .bind(material.file_url)
        .bind(material.file_public_id)
        .bind(material.video_url)
        .bind(material.is_published)
        .bind(material.deleted_at)
        .bind(material.created_at)
        .bind(material.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::String(id)), None, None, false)
            .await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
        only_published: bool,
    ) -> Result<LearningMaterial, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_query_filters(&mut sql, query, school_id, only_published)?;
        sql.push(" ORDER BY updated_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::material_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Learning material not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
        only_published: bool,
    ) -> Result<Paginated<LearningMaterial>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_sql = QueryBuilder::<Postgres>::new(
            "SELECT count(*)::BIGINT FROM learning_materials WHERE deleted_at IS NULL",
        );
        Self::push_query_filters(&mut count_sql, query, school_id, only_published)?;
        if let Some(filter) = filter.as_deref() {
            Self::push_search(&mut count_sql, filter);
        }
        let total: i64 = count_sql
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query, school_id, only_published)?;
        if let Some(filter) = filter.as_deref() {
            Self::push_search(&mut sql, filter);
        }
        sql.push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);

        let rows = sql
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let data = rows
            .into_iter()
            .map(|row| Self::material_from_row(&row))
            .collect::<Result<Vec<_>, _>>()?;
        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64) / (limit as f64)).ceil() as i64
        };

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &LearningMaterialPartial,
        file_bytes: Option<Vec<u8>>,
        file_name: Option<String>,
        _user: &AuthUserDto,
        _state: &AppState,
    ) -> Result<LearningMaterial, AppError> {
        let existing = self.find_one(Some(id), None, None, false).await?;
        let mut update_data = update.clone();

        if let (Some(bytes), Some(name)) = (file_bytes, file_name) {
            if let Some(old_public_id) = existing.file_public_id.clone() {
                CloudinaryService::delete_file(&old_public_id).await.ok();
            }

            let school_id = existing.school_id.ok_or(AppError {
                message: "School ID is required".into(),
            })?;
            let subject_id = existing.subject_id.ok_or(AppError {
                message: "Subject ID is required".into(),
            })?;
            let folder = format!(
                "space-together/{}/subjects/{}",
                school_id.to_hex(),
                subject_id.to_hex()
            );

            let upload_result = CloudinaryService::upload_file(bytes, &name, &folder)
                .await
                .map_err(|e| AppError { message: e })?;
            update_data.file_url = Some(Some(upload_result.url));
            update_data.file_public_id = Some(Some(upload_result.public_id));
        }

        let id = Self::id_to_string(id)?;
        let mut sql =
            QueryBuilder::<Postgres>::new("UPDATE learning_materials SET updated_at = now()");

        if let Some(school_id) = &update_data.school_id {
            sql.push(", school_id = ")
                .push_bind(school_id.as_ref().map(|id| id.to_hex()));
        }
        if let Some(class_id) = &update_data.class_id {
            sql.push(", class_id = ")
                .push_bind(class_id.as_ref().map(|id| id.to_hex()));
        }
        if let Some(subject_id) = &update_data.subject_id {
            let subject_id = subject_id.as_ref().map(|id| id.to_hex());
            sql.push(", subject_id = ")
                .push_bind(subject_id.clone())
                .push(", class_subject_id = ")
                .push_bind(subject_id);
        }
        if let Some(uploaded_by) = &update_data.uploaded_by {
            sql.push(", uploaded_by = ")
                .push_bind(uploaded_by.as_ref().map(|id| id.to_hex()));
        }
        if let Some(title) = &update_data.title {
            sql.push(", title = ").push_bind(title);
        }
        if let Some(description) = &update_data.description {
            sql.push(", description = ").push_bind(description);
        }
        if let Some(material_type) = &update_data.material_type {
            sql.push(", material_type = ")
                .push_bind(Self::material_type_to_string(material_type));
        }
        if let Some(file_url) = &update_data.file_url {
            sql.push(", file_url = ")
                .push_bind(file_url)
                .push(", url = ")
                .push_bind(file_url);
        }
        if let Some(file_public_id) = &update_data.file_public_id {
            sql.push(", file_public_id = ").push_bind(file_public_id);
        }
        if let Some(video_url) = &update_data.video_url {
            sql.push(", video_url = ").push_bind(video_url);
        }
        if let Some(is_published) = update_data.is_published {
            sql.push(", is_published = ").push_bind(is_published);
        }
        if let Some(deleted_at) = update_data.deleted_at {
            sql.push(", deleted_at = ").push_bind(deleted_at);
        }
        if let Some(created_at) = update_data.created_at {
            sql.push(", created_at = ").push_bind(created_at);
        }
        if let Some(updated_at) = update_data.updated_at {
            sql.push(", updated_at = ").push_bind(updated_at);
        }

        sql.push(" WHERE id = ")
            .push_bind(&id)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::String(id)), None, None, false)
            .await
    }

    pub async fn delete(
        &self,
        id: &IdType,
        _user: &AuthUserDto,
        _state: &AppState,
    ) -> Result<LearningMaterial, AppError> {
        let material = self.find_one(Some(id), None, None, false).await?;
        sqlx::query(
            "UPDATE learning_materials SET deleted_at = now(), updated_at = now() WHERE id = $1",
        )
        .bind(Self::id_to_string(id)?)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(material)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
        only_published: bool,
    ) -> Result<Paginated<LearningMaterialWithRelations>, AppError> {
        let page = self
            .get_all(filter, limit, skip, query, school_id, only_published)
            .await?;
        Ok(Paginated {
            data: page
                .data
                .into_iter()
                .map(|learning_material| LearningMaterialWithRelations {
                    learning_material,
                    uploader: None,
                    school: None,
                    class: None,
                    subject: None,
                })
                .collect(),
            total: page.total,
            total_pages: page.total_pages,
            current_page: page.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
        only_published: bool,
    ) -> Result<LearningMaterialWithRelations, AppError> {
        let learning_material = self.find_one(id, query, school_id, only_published).await?;
        Ok(LearningMaterialWithRelations {
            learning_material,
            uploader: None,
            school: None,
            class: None,
            subject: None,
        })
    }

    pub async fn count_materials(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
        only_published: bool,
    ) -> Result<u64, AppError> {
        Ok(self
            .get_all(filter, Some(1), Some(0), query, school_id, only_published)
            .await?
            .total as u64)
    }
}
