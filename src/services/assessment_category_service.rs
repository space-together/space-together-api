use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        assessment_category::{AssessmentCategory, AssessmentCategoryPartial},
        common_details::Paginated,
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct AssessmentCategoryQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub class_subject_id: Option<String>,
    pub education_year_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl AssessmentCategoryQuery {
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
            class_subject_id: query.class_id.clone(),
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

pub struct AssessmentCategoryService {
    pub pool: PgPool,
}

impl AssessmentCategoryService {
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

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, class_subject_id, education_year_id, name, code,
               COALESCE(weight_percentage, weight, 0)::DOUBLE PRECISION AS weight_percentage,
               description, created_by, created_at, updated_at, deleted_at
        FROM assessment_categories
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a AssessmentCategoryQuery>,
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
        if let Some(class_subject_id) = &query.class_subject_id {
            parse_object_id_value(class_subject_id)?;
            sql.push(" AND class_subject_id = ")
                .push_bind(class_subject_id);
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
                "school_id" | "class_subject_id" | "education_year_id" | "created_by" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND ").push(field).push(" = ").push_bind(value);
                }
                "name" | "code" => {
                    sql.push(" AND lower(")
                        .push(field)
                        .push(") = lower(")
                        .push_bind(value)
                        .push(")");
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported assessment category filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(code, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(description, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    fn category_from_row(row: PgRow) -> Result<AssessmentCategory, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let weight = row
            .try_get::<Option<f64>, _>("weight_percentage")
            .ok()
            .flatten()
            .unwrap_or_default();

        Ok(AssessmentCategory {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            class_subject_id: Self::parse_oid_opt(
                row.try_get("class_subject_id").ok().flatten(),
                "class_subject_id",
            )?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            name: row.try_get("name").map_err(Self::db_error)?,
            code: row
                .try_get::<Option<String>, _>("code")
                .ok()
                .flatten()
                .unwrap_or_default(),
            weight_percentage: weight,
            description: row.try_get("description").ok().flatten(),
            created_by: Self::parse_oid_opt(
                row.try_get("created_by").ok().flatten(),
                "created_by",
            )?,
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
            is_deleted: row
                .try_get::<Option<chrono::DateTime<Utc>>, _>("deleted_at")
                .ok()
                .flatten()
                .is_some(),
        })
    }

    pub async fn create(
        &self,
        category: AssessmentCategory,
    ) -> Result<AssessmentCategory, AppError> {
        self.ensure_indexes().await?;
        self.validate_total_weight(
            &category.class_subject_id,
            &category.education_year_id,
            category.weight_percentage,
            None,
        )
        .await?;

        let school_id = category
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let id = category
            .id
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO assessment_categories (
              id, school_id, class_subject_id, education_year_id, name, code,
              weight, weight_percentage, description, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, COALESCE($10, now()), COALESCE($11, now()))
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(category.class_subject_id.as_ref().map(|id| id.to_hex()))
        .bind(category.education_year_id.as_ref().map(|id| id.to_hex()))
        .bind(&category.name)
        .bind(&category.code)
        .bind(category.weight_percentage)
        .bind(&category.description)
        .bind(category.created_by.as_ref().map(|id| id.to_hex()))
        .bind(category.created_at)
        .bind(category.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(&IdType::from_string(id), None).await
    }

    pub async fn find_one(
        &self,
        id: &IdType,
        query: Option<AssessmentCategoryQuery>,
    ) -> Result<AssessmentCategory, AppError> {
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
            Some(row) => Self::category_from_row(row),
            None => Err(AppError {
                message: "Assessment category not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<AssessmentCategoryQuery>,
    ) -> Result<Paginated<AssessmentCategory>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM assessment_categories WHERE deleted_at IS NULL",
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
        let data = rows
            .into_iter()
            .map(Self::category_from_row)
            .collect::<Result<Vec<_>, _>>()?;

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
        update: &AssessmentCategoryPartial,
    ) -> Result<AssessmentCategory, AppError> {
        if let Some(new_weight) = update.weight_percentage {
            let existing = self.find_one(id, None).await?;
            self.validate_total_weight(
                &existing.class_subject_id,
                &existing.education_year_id,
                new_weight,
                Some(id),
            )
            .await?;
        }

        let id_string = Self::id_to_string(id)?;
        let mut sql =
            QueryBuilder::<Postgres>::new("UPDATE assessment_categories SET updated_at = now()");
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
        set_required!(code, "code");
        set_optional!(description, "description");

        if let Some(value) = update.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.class_subject_id {
            sql.push(", class_subject_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.education_year_id {
            sql.push(", education_year_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.created_by {
            sql.push(", created_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.weight_percentage {
            sql.push(", weight = ")
                .push_bind(value)
                .push(", weight_percentage = ")
                .push_bind(value);
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

    pub async fn delete(&self, id: &IdType) -> Result<AssessmentCategory, AppError> {
        let category = self.find_one(id, None).await?;
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE assessment_categories SET deleted_at = now(), updated_at = now() WHERE id = $1",
        )
        .bind(id_string)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(category)
    }

    pub async fn validate_total_weight(
        &self,
        class_subject_id: &Option<ObjectId>,
        education_year_id: &Option<ObjectId>,
        new_weight: f64,
        exclude_id: Option<&IdType>,
    ) -> Result<(), AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(
            "SELECT COALESCE(SUM(COALESCE(weight_percentage, weight, 0)), 0)::DOUBLE PRECISION FROM assessment_categories WHERE deleted_at IS NULL",
        );
        match class_subject_id {
            Some(id) => {
                sql.push(" AND class_subject_id = ").push_bind(id.to_hex());
            }
            None => {
                sql.push(" AND class_subject_id IS NULL");
            }
        };
        match education_year_id {
            Some(id) => {
                sql.push(" AND education_year_id = ").push_bind(id.to_hex());
            }
            None => {
                sql.push(" AND education_year_id IS NULL");
            }
        };
        if let Some(id) = exclude_id {
            sql.push(" AND id <> ").push_bind(Self::id_to_string(id)?);
        }

        let current: f64 = sql
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let total_weight = current + new_weight;

        if total_weight > 100.0 {
            return Err(AppError {
                message: format!(
                    "Total weight percentage exceeds 100%. Current total: {:.2}%",
                    total_weight
                ),
            });
        }

        Ok(())
    }

    pub async fn get_total_weight(
        &self,
        class_subject_id: &ObjectId,
        education_year_id: &ObjectId,
    ) -> Result<f64, AppError> {
        let total: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(COALESCE(weight_percentage, weight, 0)), 0)::DOUBLE PRECISION
            FROM assessment_categories
            WHERE class_subject_id = $1 AND education_year_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(class_subject_id.to_hex())
        .bind(education_year_id.to_hex())
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(total)
    }
}
