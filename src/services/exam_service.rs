use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        exam::{Exam, ExamPartial, ExamStatus, ExamType},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct ExamQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub class_id: Option<String>,
    pub education_year_id: Option<String>,
    pub term_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl ExamQuery {
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
            education_year_id: query.education_year_id.clone(),
            term_id: query.term_id.clone(),
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

pub struct ExamService {
    pub pool: PgPool,
}

impl ExamService {
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
        SELECT id, school_id, education_year_id, term_id, class_id, name, description,
               exam_type, status, starts_at, ends_at, created_by, created_at, updated_at, deleted_at
        FROM exams
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a ExamQuery>,
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
        if let Some(class_id) = &query.class_id {
            parse_object_id_value(class_id)?;
            sql.push(" AND class_id = ").push_bind(class_id);
        }
        if let Some(education_year_id) = &query.education_year_id {
            parse_object_id_value(education_year_id)?;
            sql.push(" AND education_year_id = ")
                .push_bind(education_year_id);
        }
        if let Some(term_id) = &query.term_id {
            sql.push(" AND term_id = ").push_bind(term_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "school_id" | "class_id" | "education_year_id" | "created_by" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND ").push(field).push(" = ").push_bind(value);
                }
                "term_id" => {
                    sql.push(" AND term_id = ").push_bind(value);
                }
                "status" => {
                    sql.push(" AND lower(status) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "exam_type" => {
                    sql.push(" AND lower(exam_type) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "name" => {
                    sql.push(" AND lower(name) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported exam filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(description, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(status, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(exam_type, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    fn exam_from_row(row: PgRow) -> Result<Exam, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Exam {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            term_id: row.try_get("term_id").ok().flatten(),
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            name: row.try_get("name").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            exam_type: Self::enum_from_string::<ExamType>(row.try_get("exam_type").ok().flatten())
                .unwrap_or(ExamType::Continuous),
            status: Self::enum_from_string::<ExamStatus>(row.try_get("status").ok().flatten())
                .unwrap_or(ExamStatus::Draft),
            start_date: row.try_get("starts_at").ok().unwrap_or_else(Utc::now),
            end_date: row.try_get("ends_at").ok().unwrap_or_else(Utc::now),
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

    pub async fn create(&self, exam: Exam) -> Result<Exam, AppError> {
        self.ensure_indexes().await?;
        let school_id = exam
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let id = exam.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO exams (
              id, school_id, education_year_id, term_id, class_id, name, description,
              exam_type, status, starts_at, ends_at, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, COALESCE($13, now()), COALESCE($14, now()))
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(exam.education_year_id.as_ref().map(|id| id.to_hex()))
        .bind(&exam.term_id)
        .bind(exam.class_id.as_ref().map(|id| id.to_hex()))
        .bind(&exam.name)
        .bind(&exam.description)
        .bind(Self::enum_to_string(&exam.exam_type))
        .bind(Self::enum_to_string(&exam.status))
        .bind(exam.start_date)
        .bind(exam.end_date)
        .bind(exam.created_by.as_ref().map(|id| id.to_hex()))
        .bind(exam.created_at)
        .bind(exam.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(&IdType::from_string(id), None).await
    }

    pub async fn find_one(&self, id: &IdType, query: Option<ExamQuery>) -> Result<Exam, AppError> {
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
            Some(row) => Self::exam_from_row(row),
            None => Err(AppError {
                message: "Exam not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ExamQuery>,
    ) -> Result<Paginated<Exam>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM exams WHERE deleted_at IS NULL");
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
        let data = rows
            .into_iter()
            .map(Self::exam_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &ExamPartial) -> Result<Exam, AppError> {
        let id_string = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE exams SET updated_at = now()");
        let mut touched = false;

        macro_rules! set_required {
            ($field:ident, $column:literal) => {
                if let Some(value) = update.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }
        macro_rules! set_optional {
            ($field:ident, $column:literal) => {
                if let Some(value) = update.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }

        set_required!(name, "name");
        set_optional!(description, "description");
        set_optional!(term_id, "term_id");

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
        if let Some(value) = update.class_id {
            sql.push(", class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.created_by {
            sql.push(", created_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.exam_type.clone() {
            sql.push(", exam_type = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update.status.clone() {
            sql.push(", status = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update.start_date {
            sql.push(", starts_at = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update.end_date {
            sql.push(", ends_at = ").push_bind(value);
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

        if !touched {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        sql.push(" WHERE id = ")
            .push_bind(&id_string)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one(id, None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Exam, AppError> {
        let exam = self.find_one(id, None).await?;
        let id_string = Self::id_to_string(id)?;
        sqlx::query("UPDATE exams SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(id_string)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(exam)
    }

    pub async fn publish(&self, id: &IdType) -> Result<Exam, AppError> {
        let id_string = Self::id_to_string(id)?;
        sqlx::query("UPDATE exams SET status = 'Published', updated_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id_string)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one(id, None).await
    }

    pub async fn count_exams(
        &self,
        filter: Option<String>,
        query: Option<ExamQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }
}
