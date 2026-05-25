use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        grading_scale::{GradeBoundary, GradingScale, GradingScalePartial, GradingType},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct GradingScaleQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub education_year_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl GradingScaleQuery {
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
            education_year_id: query.education_year_id.clone(),
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

pub struct GradingScaleService {
    pub pool: PgPool,
}

impl GradingScaleService {
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

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, education_year_id, name, grading_type, is_active,
               created_by, created_at, updated_at, deleted_at
        FROM grading_scales
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a GradingScaleQuery>,
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
        if let Some(education_year_id) = &query.education_year_id {
            parse_object_id_value(education_year_id)?;
            sql.push(" AND education_year_id = ")
                .push_bind(education_year_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "school_id" | "education_year_id" | "created_by" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND ").push(field).push(" = ").push_bind(value);
                }
                "name" | "grading_type" => {
                    sql.push(" AND lower(")
                        .push(field)
                        .push(") = lower(")
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
                        message: format!("Unsupported grading scale filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(grading_type, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    async fn fetch_boundaries(&self, scale_id: &str) -> Result<Vec<GradeBoundary>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT grade, min_score, max_score, gpa_value, description
            FROM grading_scale_boundaries
            WHERE grading_scale_id = $1
            ORDER BY position ASC, min_score DESC
            "#,
        )
        .bind(scale_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(GradeBoundary {
                    grade: row.try_get("grade").map_err(Self::db_error)?,
                    min_score: row.try_get("min_score").map_err(Self::db_error)?,
                    max_score: row.try_get("max_score").map_err(Self::db_error)?,
                    gpa_value: row.try_get("gpa_value").ok().flatten(),
                    description: row.try_get("description").ok().flatten(),
                })
            })
            .collect()
    }

    async fn sync_boundaries(
        &self,
        scale_id: &str,
        boundaries: &[GradeBoundary],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM grading_scale_boundaries WHERE grading_scale_id = $1")
            .bind(scale_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, boundary) in boundaries.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO grading_scale_boundaries (
                  id, grading_scale_id, grade, min_score, max_score, gpa_value, description, position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(Self::new_id())
            .bind(scale_id)
            .bind(&boundary.grade)
            .bind(boundary.min_score)
            .bind(boundary.max_score)
            .bind(boundary.gpa_value)
            .bind(&boundary.description)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }
        Ok(())
    }

    async fn scale_from_row(&self, row: PgRow) -> Result<GradingScale, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(GradingScale {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            name: row.try_get("name").map_err(Self::db_error)?,
            grading_type: Self::enum_from_string::<GradingType>(
                row.try_get("grading_type").ok().flatten(),
            )
            .unwrap_or(GradingType::Letter),
            grade_boundaries: self.fetch_boundaries(&id).await?,
            is_active: row.try_get("is_active").ok().unwrap_or(false),
            created_by: Self::parse_oid_opt(
                row.try_get("created_by").ok().flatten(),
                "created_by",
            )?,
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
            is_deleted: row
                .try_get::<Option<DateTime<Utc>>, _>("deleted_at")
                .ok()
                .flatten()
                .is_some(),
        })
    }

    pub async fn create(&self, scale: GradingScale) -> Result<GradingScale, AppError> {
        self.ensure_indexes().await?;
        let school_id = scale
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let id = scale.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO grading_scales (
              id, school_id, education_year_id, name, grading_type, is_active,
              created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, now()), COALESCE($9, now()))
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(scale.education_year_id.as_ref().map(|id| id.to_hex()))
        .bind(&scale.name)
        .bind(Self::enum_to_string(&scale.grading_type))
        .bind(scale.is_active)
        .bind(scale.created_by.as_ref().map(|id| id.to_hex()))
        .bind(scale.created_at)
        .bind(scale.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_boundaries(&id, &scale.grade_boundaries).await?;
        if scale.is_active {
            self.deactivate_other_scales(&id).await?;
        }
        self.find_one(&IdType::from_string(id), None).await
    }

    pub async fn find_one(
        &self,
        id: &IdType,
        query: Option<GradingScaleQuery>,
    ) -> Result<GradingScale, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query.as_ref())?;
        sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        sql.push(" ORDER BY updated_at DESC LIMIT 1");
        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        match row {
            Some(row) => self.scale_from_row(row).await,
            None => Err(AppError {
                message: "Grading scale not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<GradingScaleQuery>,
    ) -> Result<Paginated<GradingScale>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM grading_scales WHERE deleted_at IS NULL",
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
            data.push(self.scale_from_row(row).await?);
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
        update: &GradingScalePartial,
    ) -> Result<GradingScale, AppError> {
        let id_string = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE grading_scales SET updated_at = now()");
        let mut touched = false;

        if let Some(value) = update.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.education_year_id {
            sql.push(", education_year_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.name.clone() {
            sql.push(", name = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update.grading_type.clone() {
            sql.push(", grading_type = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update.is_active {
            sql.push(", is_active = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update.created_by {
            sql.push(", created_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.is_deleted {
            sql.push(", deleted_at = ");
            if value {
                sql.push("now()");
            } else {
                sql.push("NULL");
            }
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
        if let Some(boundaries) = &update.grade_boundaries {
            self.sync_boundaries(&id_string, boundaries).await?;
        }
        if let Some(true) = update.is_active {
            self.deactivate_other_scales(&id_string).await?;
        }
        if !touched && update.grade_boundaries.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.find_one(id, None).await
    }

    async fn deactivate_other_scales(&self, id: &str) -> Result<(), AppError> {
        let scale = self.find_one(&IdType::from_string(id), None).await?;
        let school_id = scale.school_id.ok_or(AppError {
            message: "Grading scale has no school_id".into(),
        })?;
        let education_year_id = scale.education_year_id.ok_or(AppError {
            message: "Grading scale has no education_year_id".into(),
        })?;

        sqlx::query(
            r#"
            UPDATE grading_scales
            SET is_active = false, updated_at = now()
            WHERE school_id = $1 AND education_year_id = $2 AND id <> $3 AND deleted_at IS NULL
            "#,
        )
        .bind(school_id.to_hex())
        .bind(education_year_id.to_hex())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn activate(&self, id: &IdType) -> Result<GradingScale, AppError> {
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE grading_scales SET is_active = true, updated_at = now() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(&id_string)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        self.deactivate_other_scales(&id_string).await?;
        self.find_one(id, None).await
    }

    pub async fn get_active_scale(
        &self,
        school_id: &ObjectId,
        education_year_id: &ObjectId,
    ) -> Result<Option<GradingScale>, AppError> {
        let query = GradingScaleQuery {
            school_id: Some(school_id.to_hex()),
            education_year_id: Some(education_year_id.to_hex()),
            field_values: vec![("is_active".into(), "true".into())],
            ..GradingScaleQuery::default()
        };
        let page = self.get_all(None, Some(1), Some(0), Some(query)).await?;
        Ok(page.data.into_iter().next())
    }

    pub fn calculate_grade(&self, scale: &GradingScale, percentage: f64) -> String {
        for boundary in &scale.grade_boundaries {
            if percentage >= boundary.min_score && percentage <= boundary.max_score {
                return boundary.grade.clone();
            }
        }
        "N/A".to_string()
    }
}
