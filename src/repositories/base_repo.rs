use crate::{
    domain::common_details::Paginated,
    errors::AppError,
    models::{id_model::IdType, mongo_model::CountDoc},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{postgres::PgRow, FromRow, PgPool, Postgres, QueryBuilder};

#[derive(Clone, Debug)]
pub enum SqlValue {
    Text(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Timestamp(DateTime<Utc>),
    Null,
}

impl From<&str> for SqlValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for SqlValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<bool> for SqlValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for SqlValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<i32> for SqlValue {
    fn from(value: i32) -> Self {
        Self::I64(value as i64)
    }
}

impl From<f64> for SqlValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<DateTime<Utc>> for SqlValue {
    fn from(value: DateTime<Utc>) -> Self {
        Self::Timestamp(value)
    }
}

#[derive(Clone, Debug)]
pub enum SqlClause {
    EqText {
        column: &'static str,
        value: String,
    },
    InText {
        column: &'static str,
        values: Vec<String>,
    },
    IsNull {
        column: &'static str,
    },
    IsNotNull {
        column: &'static str,
    },
}

#[derive(Clone, Debug, Default)]
pub struct SqlFilter {
    clauses: Vec<SqlClause>,
}

impl SqlFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn by_id(id: &IdType) -> Result<Self, AppError> {
        Ok(Self::new().eq_text("id", id.as_string()))
    }

    pub fn eq_text(mut self, column: &'static str, value: impl Into<String>) -> Self {
        self.clauses.push(SqlClause::EqText {
            column,
            value: value.into(),
        });
        self
    }

    pub fn in_text(mut self, column: &'static str, values: Vec<String>) -> Self {
        self.clauses.push(SqlClause::InText { column, values });
        self
    }

    pub fn is_null(mut self, column: &'static str) -> Self {
        self.clauses.push(SqlClause::IsNull { column });
        self
    }

    pub fn is_not_null(mut self, column: &'static str) -> Self {
        self.clauses.push(SqlClause::IsNotNull { column });
        self
    }

    fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}

pub struct BaseRepository {
    pub pool: PgPool,
    table: &'static str,
    has_deleted_at: bool,
}

impl BaseRepository {
    pub fn new(pool: &PgPool, table: &'static str) -> Self {
        Self {
            pool: pool.clone(),
            table,
            has_deleted_at: true,
        }
    }

    pub fn without_soft_delete(mut self) -> Self {
        self.has_deleted_at = false;
        self
    }

    fn db_error(error: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", error),
        }
    }

    fn push_value<'a>(query: &mut QueryBuilder<'a, Postgres>, value: &'a SqlValue) {
        match value {
            SqlValue::Text(value) => {
                query.push_bind(value);
            }
            SqlValue::I64(value) => {
                query.push_bind(*value);
            }
            SqlValue::F64(value) => {
                query.push_bind(*value);
            }
            SqlValue::Bool(value) => {
                query.push_bind(*value);
            }
            SqlValue::Timestamp(value) => {
                query.push_bind(*value);
            }
            SqlValue::Null => {
                query.push("NULL");
            }
        }
    }

    fn push_filter<'a>(query: &mut QueryBuilder<'a, Postgres>, filter: &'a SqlFilter) {
        for clause in &filter.clauses {
            match clause {
                SqlClause::EqText { column, value } => {
                    query
                        .push(" AND ")
                        .push(*column)
                        .push(" = ")
                        .push_bind(value);
                }
                SqlClause::InText { column, values } => {
                    if values.is_empty() {
                        query.push(" AND false");
                    } else {
                        query.push(" AND ").push(*column).push(" IN (");
                        let mut separated = query.separated(", ");
                        for value in values {
                            separated.push_bind(value);
                        }
                        separated.push_unseparated(")");
                    }
                }
                SqlClause::IsNull { column } => {
                    query.push(" AND ").push(*column).push(" IS NULL");
                }
                SqlClause::IsNotNull { column } => {
                    query.push(" AND ").push(*column).push(" IS NOT NULL");
                }
            }
        }
    }

    fn push_search<'a>(
        query: &mut QueryBuilder<'a, Postgres>,
        filter: Option<&'a str>,
        searchable_fields: &[&'static str],
    ) {
        let Some(filter) = filter else {
            return;
        };
        if filter.trim().is_empty() || searchable_fields.is_empty() {
            return;
        }

        let search = format!("%{}%", filter.to_lowercase());
        query.push(" AND (");
        let mut separated = query.separated(" OR ");
        for field in searchable_fields {
            separated
                .push("lower(coalesce(")
                .push(*field)
                .push("::text, '')) LIKE ")
                .push_bind(search.clone());
        }
        separated.push_unseparated(")");
    }

    fn base_select(&self) -> String {
        format!("SELECT * FROM {} WHERE true", self.table)
    }

    fn base_count(&self) -> String {
        format!("SELECT count(*) FROM {} WHERE true", self.table)
    }

    fn base_delete(&self) -> String {
        format!("DELETE FROM {} WHERE true", self.table)
    }

    fn push_soft_delete_guard(&self, query: &mut QueryBuilder<'_, Postgres>) {
        if self.has_deleted_at {
            query.push(" AND deleted_at IS NULL");
        }
    }

    pub async fn create<T>(&self, values: &[(&'static str, SqlValue)]) -> Result<T, AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        if values.is_empty() {
            return Err(AppError {
                message: "No valid fields to create".into(),
            });
        }

        let mut query = QueryBuilder::<Postgres>::new("INSERT INTO ");
        query.push(self.table).push(" (");
        let mut columns = query.separated(", ");
        for (column, _) in values {
            columns.push(*column);
        }
        columns.push_unseparated(") VALUES (");
        let mut bind_values = query.separated(", ");
        for (_, value) in values {
            match value {
                SqlValue::Text(value) => {
                    bind_values.push_bind(value);
                }
                SqlValue::I64(value) => {
                    bind_values.push_bind(*value);
                }
                SqlValue::F64(value) => {
                    bind_values.push_bind(*value);
                }
                SqlValue::Bool(value) => {
                    bind_values.push_bind(*value);
                }
                SqlValue::Timestamp(value) => {
                    bind_values.push_bind(*value);
                }
                SqlValue::Null => {
                    bind_values.push("NULL");
                }
            }
        }
        bind_values.push_unseparated(") RETURNING *");

        query
            .build_query_as::<T>()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)
    }

    pub async fn find_one<T>(&self, filter: SqlFilter) -> Result<Option<T>, AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        let mut query = QueryBuilder::<Postgres>::new(self.base_select());
        self.push_soft_delete_guard(&mut query);
        Self::push_filter(&mut query, &filter);
        query.push(" LIMIT 1");

        query
            .build_query_as::<T>()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)
    }

    pub async fn get_all<T>(
        &self,
        filter: Option<String>,
        searchable_fields: &[&'static str],
        limit: Option<i64>,
        skip: Option<i64>,
        extra_filter: Option<SqlFilter>,
    ) -> Result<(Vec<T>, i64, i64, i64), AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        let limit_value = limit.unwrap_or(50).max(1);
        let skip_value = skip.unwrap_or(0).max(0);
        let empty_filter = SqlFilter::new();
        let extra_filter = extra_filter.as_ref().unwrap_or(&empty_filter);

        let mut count_query = QueryBuilder::<Postgres>::new(self.base_count());
        self.push_soft_delete_guard(&mut count_query);
        Self::push_search(&mut count_query, filter.as_deref(), searchable_fields);
        if !extra_filter.is_empty() {
            Self::push_filter(&mut count_query, extra_filter);
        }

        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut query = QueryBuilder::<Postgres>::new(self.base_select());
        self.push_soft_delete_guard(&mut query);
        Self::push_search(&mut query, filter.as_deref(), searchable_fields);
        if !extra_filter.is_empty() {
            Self::push_filter(&mut query, extra_filter);
        }
        query
            .push(" ORDER BY updated_at DESC LIMIT ")
            .push_bind(limit_value)
            .push(" OFFSET ")
            .push_bind(skip_value);

        let data = query
            .build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let current_page = skip_value / limit_value + 1;
        let total_pages = if total > 0 {
            ((total as f64) / (limit_value as f64)).ceil() as i64
        } else {
            1
        };

        Ok((data, total, total_pages, current_page))
    }

    pub async fn update_one_and_fetch<T>(
        &self,
        id: &IdType,
        values: &[(&'static str, SqlValue)],
    ) -> Result<T, AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        if values.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let mut query = QueryBuilder::<Postgres>::new("UPDATE ");
        query.push(self.table).push(" SET updated_at = now()");
        for (column, value) in values {
            query.push(", ").push(*column).push(" = ");
            Self::push_value(&mut query, value);
        }
        query.push(" WHERE id = ").push_bind(id.as_string());
        self.push_soft_delete_guard(&mut query);
        query.push(" RETURNING *");

        query
            .build_query_as::<T>()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .ok_or(AppError {
                message: "Record not found".into(),
            })
    }

    pub async fn update_many_and_fetch<T>(
        &self,
        filter: SqlFilter,
        values: &[(&'static str, SqlValue)],
    ) -> Result<Vec<T>, AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        if filter.is_empty() {
            return Err(AppError {
                message: "Update filter cannot be empty".into(),
            });
        }
        if values.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        let mut query = QueryBuilder::<Postgres>::new("UPDATE ");
        query.push(self.table).push(" SET updated_at = now()");
        for (column, value) in values {
            query.push(", ").push(*column).push(" = ");
            Self::push_value(&mut query, value);
        }
        query.push(" WHERE true");
        self.push_soft_delete_guard(&mut query);
        Self::push_filter(&mut query, &filter);
        query.push(" RETURNING *");

        query
            .build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)
    }

    pub async fn delete_one(&self, id: &IdType) -> Result<(), AppError> {
        let mut query = QueryBuilder::<Postgres>::new(self.base_delete());
        query.push(" AND id = ").push_bind(id.as_string());

        let result = query
            .build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        if result.rows_affected() == 0 {
            Err(AppError {
                message: "Record not found".into(),
            })
        } else {
            Ok(())
        }
    }

    pub async fn delete_many(&self, filter: SqlFilter) -> Result<(), AppError> {
        if filter.is_empty() {
            return Err(AppError {
                message: "Delete filter cannot be empty".into(),
            });
        }

        let mut query = QueryBuilder::<Postgres>::new(self.base_delete());
        Self::push_filter(&mut query, &filter);

        query
            .build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        searchable_fields: &[&'static str],
        extra_filter: Option<SqlFilter>,
    ) -> Result<CountDoc, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(self.base_count());
        self.push_soft_delete_guard(&mut query);
        Self::push_search(&mut query, filter.as_deref(), searchable_fields);
        if let Some(extra_filter) = &extra_filter {
            Self::push_filter(&mut query, extra_filter);
        }

        let count: i64 = query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(CountDoc {
            count: count.max(0) as u64,
        })
    }

    pub async fn get_all_paginated<T>(
        &self,
        filter: Option<String>,
        searchable_fields: &[&'static str],
        limit: Option<i64>,
        skip: Option<i64>,
        extra_filter: Option<SqlFilter>,
    ) -> Result<Paginated<T>, AppError>
    where
        for<'r> T: FromRow<'r, PgRow> + Send + Unpin,
    {
        let (data, total, total_pages, current_page) = self
            .get_all(filter, searchable_fields, limit, skip, extra_filter)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub fn serialize_for_response<T: Serialize>(value: &T) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(value).map_err(|e| AppError {
            message: format!("Failed to serialize response: {}", e),
        })
    }
}
