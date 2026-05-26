use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        audit_log::{AuditLog, AuditLogWithRelations, AuditSeverity, RequestMeta},
        auth_user::AuthUserDto,
        common_details::{Paginated, UserRole},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    utils::{object_id::parse_object_id_value, object_id::ObjectId},
};

pub struct AuditLogService {
    pub pool: PgPool,
}

impl AuditLogService {
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

    fn fallback_id() -> &'static str {
        "000000000000000000000000"
    }

    fn id_to_string(id: &IdType) -> Result<String, AppError> {
        Ok(IdType::to_object_id(id)?.to_hex())
    }

    fn parse_oid(raw: &str, field: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(raw).map_err(|e| AppError {
            message: format!("Invalid {} ObjectId-compatible ID: {}", field, e),
        })
    }

    fn role_to_string(role: &UserRole) -> String {
        role.to_string()
    }

    fn role_from_string(raw: Option<String>) -> UserRole {
        match raw
            .unwrap_or_else(|| "STUDENT".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "TEACHER" => UserRole::TEACHER,
            "ADMIN" => UserRole::ADMIN,
            "SCHOOLSTAFF" => UserRole::SCHOOLSTAFF,
            "PARENT" => UserRole::PARENT,
            _ => UserRole::STUDENT,
        }
    }

    fn severity_to_string(severity: AuditSeverity) -> &'static str {
        match severity {
            AuditSeverity::INFO => "INFO",
            AuditSeverity::WARNING => "WARNING",
            AuditSeverity::CRITICAL => "CRITICAL",
        }
    }

    fn severity_from_string(raw: Option<String>) -> AuditSeverity {
        match raw
            .unwrap_or_else(|| "INFO".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "WARNING" => AuditSeverity::WARNING,
            "CRITICAL" => AuditSeverity::CRITICAL,
            _ => AuditSeverity::INFO,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id,
               COALESCE(school_id, '000000000000000000000000') AS school_id,
               COALESCE(actor_user_id, '000000000000000000000000') AS user_id,
               actor_role AS user_role,
               action, entity_type,
               COALESCE(entity_id, '000000000000000000000000') AS entity_id,
               ip_address, user_agent, severity, created_at
        FROM audit_logs
        WHERE 1 = 1
        "#
    }

    fn audit_log_from_row(row: &sqlx::postgres::PgRow) -> Result<AuditLog, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let school_id: String = row.try_get("school_id").map_err(Self::db_error)?;
        let user_id: String = row.try_get("user_id").map_err(Self::db_error)?;
        let entity_id: String = row.try_get("entity_id").map_err(Self::db_error)?;

        Ok(AuditLog {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid(&school_id, "school_id")?,
            user_id: Self::parse_oid(&user_id, "user_id")?,
            user_role: Self::role_from_string(row.try_get("user_role").ok().flatten()),
            action: row.try_get("action").map_err(Self::db_error)?,
            entity_type: row.try_get("entity_type").map_err(Self::db_error)?,
            entity_id: Self::parse_oid(&entity_id, "entity_id")?,
            metadata: None,
            ip_address: row.try_get("ip_address").ok().flatten(),
            user_agent: row.try_get("user_agent").ok().flatten(),
            severity: Self::severity_from_string(row.try_get("severity").ok().flatten()),
            created_at: row
                .try_get::<Option<DateTime<Utc>>, _>("created_at")
                .ok()
                .flatten()
                .unwrap_or_else(Utc::now),
        })
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
        school_id: Option<&'a str>,
    ) -> Result<(), AppError> {
        if let Some(school_id) = school_id {
            Self::parse_oid(school_id, "school_id")?;
            sql.push(" AND school_id = ").push_bind(school_id);
        }

        let Some(query) = query else {
            return Ok(());
        };

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
                "user_id" | "actor_user_id" => {
                    Self::parse_oid(value, "user_id")?;
                    sql.push(" AND actor_user_id = ").push_bind(value);
                }
                "entity_id" => {
                    Self::parse_oid(value, "entity_id")?;
                    sql.push(" AND entity_id = ").push_bind(value);
                }
                "entity_type" => {
                    sql.push(" AND lower(entity_type) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                "action" => {
                    sql.push(" AND lower(action) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                "severity" => {
                    sql.push(" AND lower(severity) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                "user_role" | "actor_role" => {
                    sql.push(" AND lower(actor_role) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, filter: &'a str) {
        let search = format!("%{}%", filter.to_ascii_lowercase());
        sql.push(" AND (lower(action) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(entity_type) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(id) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(actor_user_id, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(school_id, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(entity_id, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(severity, '')) LIKE ")
            .push_bind(search)
            .push(")");
    }

    pub async fn log_event(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        action: &str,
        entity_type: &str,
        entity_id: ObjectId,
        _metadata: Option<serde_json::Value>,
        request_meta: Option<RequestMeta>,
        severity: Option<AuditSeverity>,
    ) -> Result<(), AppError> {
        let result = self
            .log_event_internal(
                school_id,
                user,
                action,
                entity_type,
                entity_id,
                request_meta,
                severity,
            )
            .await;

        if let Err(e) = result {
            eprintln!("Audit log failed: {:?}", e);
        }

        Ok(())
    }

    async fn log_event_internal(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        action: &str,
        entity_type: &str,
        entity_id: ObjectId,
        request_meta: Option<RequestMeta>,
        severity: Option<AuditSeverity>,
    ) -> Result<(), AppError> {
        self.ensure_indexes().await?;
        let user_id = parse_object_id_value(&user.id)?;
        let role = user
            .role
            .clone()
            .unwrap_or(crate::domain::common_details::UserRole::STUDENT);
        let severity = severity.unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
              id, school_id, actor_user_id, actor_role, entity_type, entity_id,
              action, severity, ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(Self::new_id())
        .bind(school_id.to_hex())
        .bind(user_id.to_hex())
        .bind(Self::role_to_string(&role))
        .bind(entity_type)
        .bind(entity_id.to_hex())
        .bind(action)
        .bind(Self::severity_to_string(severity))
        .bind(request_meta.as_ref().and_then(|m| m.ip_address.clone()))
        .bind(request_meta.as_ref().and_then(|m| m.user_agent.clone()))
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<AuditLog, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_query_filters(&mut sql, query, school_id)?;
        sql.push(" ORDER BY created_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::audit_log_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Audit log not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<AuditLog>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_sql =
            QueryBuilder::<Postgres>::new("SELECT count(*)::BIGINT FROM audit_logs WHERE 1 = 1");
        Self::push_query_filters(&mut count_sql, query, school_id)?;
        if let Some(filter) = filter.as_deref() {
            Self::push_search(&mut count_sql, filter);
        }
        let total: i64 = count_sql
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query, school_id)?;
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
            .map(|row| Self::audit_log_from_row(&row))
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

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<AuditLogWithRelations>, AppError> {
        let page = self.get_all(filter, limit, skip, query, school_id).await?;
        Ok(Paginated {
            data: page
                .data
                .into_iter()
                .map(|audit_log| AuditLogWithRelations {
                    audit_log,
                    user: None,
                    school: None,
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
    ) -> Result<AuditLogWithRelations, AppError> {
        let audit_log = self.find_one(id, query, school_id).await?;
        Ok(AuditLogWithRelations {
            audit_log,
            user: None,
            school: None,
        })
    }

    pub async fn count_audit_logs(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<u64, AppError> {
        Ok(self
            .get_all(filter, Some(1), Some(0), query, school_id)
            .await?
            .total as u64)
    }
}
