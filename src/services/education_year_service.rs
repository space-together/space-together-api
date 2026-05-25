use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        education_year::{EducationYear, EducationYearPartial, EducationYearWithOthers, Term},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct EducationYearQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl EducationYearQuery {
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

pub struct EducationYearService {
    pub pool: PgPool,
}

impl EducationYearService {
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

    fn date_to_datetime(date: NaiveDate) -> DateTime<Utc> {
        DateTime::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc)
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, curriculum_id, name, starts_on, ends_on, is_active,
               created_by, created_at, updated_at
        FROM education_years
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a EducationYearQuery>,
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
                "curriculum_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND curriculum_id = ").push_bind(value);
                }
                "label" | "name" => {
                    sql.push(" AND lower(name) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported education year filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(name, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(curriculum_id, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    async fn fetch_terms(&self, year_id: &str) -> Result<Vec<Term>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT name, term_order, starts_on, ends_on
            FROM terms
            WHERE education_year_id = $1 AND deleted_at IS NULL
            ORDER BY term_order ASC, starts_on ASC
            "#,
        )
        .bind(year_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| {
                let start: NaiveDate = row.try_get("starts_on").map_err(Self::db_error)?;
                let end: NaiveDate = row.try_get("ends_on").map_err(Self::db_error)?;
                Ok(Term {
                    name: row.try_get("name").map_err(Self::db_error)?,
                    order: row.try_get("term_order").map_err(Self::db_error)?,
                    start_date: Self::date_to_datetime(start),
                    end_date: Self::date_to_datetime(end),
                })
            })
            .collect()
    }

    async fn sync_terms(
        &self,
        year_id: &str,
        school_id: &str,
        terms: &[Term],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM terms WHERE education_year_id = $1")
            .bind(year_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for term in terms {
            sqlx::query(
                r#"
                INSERT INTO terms (
                  id, school_id, education_year_id, name, term_order, starts_on, ends_on, is_active
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, false)
                "#,
            )
            .bind(Self::new_id())
            .bind(school_id)
            .bind(year_id)
            .bind(&term.name)
            .bind(term.order)
            .bind(term.start_date.date_naive())
            .bind(term.end_date.date_naive())
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn year_from_row(&self, row: PgRow) -> Result<EducationYear, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let start: NaiveDate = row.try_get("starts_on").map_err(Self::db_error)?;
        let end: NaiveDate = row.try_get("ends_on").map_err(Self::db_error)?;
        Ok(EducationYear {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            curriculum_id: Self::parse_oid(
                &row.try_get::<String, _>("curriculum_id")
                    .map_err(Self::db_error)?,
                "curriculum_id",
            )?,
            label: row.try_get("name").map_err(Self::db_error)?,
            start_date: Self::date_to_datetime(start),
            end_date: Self::date_to_datetime(end),
            terms: self.fetch_terms(&id).await?,
            created_by: Self::parse_oid_opt(
                row.try_get("created_by").ok().flatten(),
                "created_by",
            )?,
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    pub async fn create(&self, dto: EducationYear) -> Result<EducationYear, AppError> {
        self.ensure_indexes().await?;
        let school_id = dto
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let curriculum_id = dto.curriculum_id.to_hex();

        if self
            .find_by_label_and_curriculum(&dto.label, dto.curriculum_id, Some(&school_id))
            .await
            .is_ok()
        {
            return Err(AppError {
                message: format!(
                    "Academic year {} already exists under this curriculum",
                    dto.label
                ),
            });
        }

        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO education_years (
              id, school_id, curriculum_id, name, starts_on, ends_on, is_active, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, true, $7, COALESCE($8, now()), COALESCE($9, now()))
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(&curriculum_id)
        .bind(&dto.label)
        .bind(dto.start_date.date_naive())
        .bind(dto.end_date.date_naive())
        .bind(dto.created_by.as_ref().map(|id| id.to_hex()))
        .bind(dto.created_at)
        .bind(dto.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_terms(&id, &school_id, &dto.terms).await?;
        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<EducationYearQuery>,
    ) -> Result<EducationYear, AppError> {
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
            Some(row) => self.year_from_row(row).await,
            None => Err(AppError {
                message: "Academic year not found".to_string(),
            }),
        }
    }

    pub async fn find_by_label_and_curriculum(
        &self,
        label: &str,
        curriculum_id: ObjectId,
        school_id: Option<&str>,
    ) -> Result<EducationYear, AppError> {
        let mut query = EducationYearQuery::default();
        query
            .field_values
            .push(("label".to_string(), label.to_string()));
        query
            .field_values
            .push(("curriculum_id".to_string(), curriculum_id.to_hex()));
        query.school_id = school_id.map(ToOwned::to_owned);
        self.find_one(None, Some(query)).await
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<EducationYearQuery>,
    ) -> Result<Paginated<EducationYear>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM education_years WHERE deleted_at IS NULL",
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
            data.push(self.year_from_row(row).await?);
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
        update: &EducationYearPartial,
    ) -> Result<EducationYear, AppError> {
        let existing = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        let school_id = existing.school_id.as_ref().map(|id| id.to_hex());

        if let (Some(label), Some(curriculum_id)) = (update.label.clone(), update.curriculum_id) {
            if (existing.label != label || existing.curriculum_id != curriculum_id)
                && self
                    .find_by_label_and_curriculum(&label, curriculum_id, school_id.as_deref())
                    .await
                    .is_ok()
            {
                return Err(AppError {
                    message: format!("Academic year {} already exists for this curriculum", label),
                });
            }
        }

        let mut sql =
            QueryBuilder::<Postgres>::new("UPDATE education_years SET updated_at = now()");
        let mut touched = false;

        if let Some(value) = update.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.curriculum_id {
            sql.push(", curriculum_id = ").push_bind(value.to_hex());
            touched = true;
        }
        if let Some(value) = update.label.clone() {
            sql.push(", name = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update.start_date {
            sql.push(", starts_on = ").push_bind(value.date_naive());
            touched = true;
        }
        if let Some(value) = update.end_date {
            sql.push(", ends_on = ").push_bind(value.date_naive());
            touched = true;
        }
        if let Some(value) = update.created_by {
            sql.push(", created_by = ")
                .push_bind(value.map(|id| id.to_hex()));
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
        if let Some(terms) = &update.terms {
            let school_id = existing
                .school_id
                .as_ref()
                .map(|id| id.to_hex())
                .ok_or_else(|| AppError {
                    message: "school_id is required".into(),
                })?;
            self.sync_terms(&id_string, &school_id, terms).await?;
        }
        if !touched && update.terms.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.find_one(Some(id), None).await
    }

    pub async fn delete(&self, id: &IdType, user_id: ObjectId) -> Result<EducationYear, AppError> {
        let education_year = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE education_years SET deleted_at = now(), deleted_by = $2, updated_at = now() WHERE id = $1",
        )
        .bind(id_string)
        .bind(user_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(education_year)
    }

    pub async fn restore(&self, id: &IdType) -> Result<EducationYear, AppError> {
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE education_years SET deleted_at = NULL, deleted_by = NULL, updated_at = now() WHERE id = $1",
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
        query: Option<EducationYearQuery>,
    ) -> Result<Paginated<EducationYearWithOthers>, AppError> {
        let years = self.get_all(filter, limit, skip, query).await?;
        let data = years
            .data
            .into_iter()
            .map(|academic| EducationYearWithOthers {
                academic,
                curriculum: None,
            })
            .collect();
        Ok(Paginated {
            data,
            total: years.total,
            total_pages: years.total_pages,
            current_page: years.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<EducationYearQuery>,
    ) -> Result<EducationYearWithOthers, AppError> {
        Ok(EducationYearWithOthers {
            academic: self.find_one(id, query).await?,
            curriculum: None,
        })
    }

    pub async fn count_education_years(
        &self,
        filter: Option<String>,
        query: Option<EducationYearQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub async fn get_current_year_and_term(
        &self,
        date: Option<DateTime<Utc>>,
        query: Option<EducationYearQuery>,
    ) -> Result<(EducationYear, Option<Term>), AppError> {
        let target_date = date.unwrap_or_else(Utc::now);
        let mut year_query = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut year_query, query.as_ref())?;
        year_query
            .push(" AND starts_on <= ")
            .push_bind(target_date.date_naive())
            .push(" AND ends_on >= ")
            .push_bind(target_date.date_naive())
            .push(" ORDER BY starts_on DESC LIMIT 1");

        let row = year_query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let year = match row {
            Some(row) => self.year_from_row(row).await?,
            None => {
                return Err(AppError {
                    message: "No active education year found for this date".to_string(),
                })
            }
        };

        let current_term = year
            .terms
            .iter()
            .find(|t| t.start_date <= target_date && t.end_date >= target_date)
            .cloned();

        Ok((year, current_term))
    }
}
