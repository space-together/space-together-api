use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::{
    domain::{auth_user::AuthUserDto, common_details::Paginated},
    errors::AppError,
    services::audit_log_service::AuditLogService,
    utils::object_id::ObjectId,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletedEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: Option<String>,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
    pub deleted_by_name: Option<String>,
    pub school_id: String,
}

// PG tables to check for soft-deleted records.
// Each entry: (table_name, name_column_or_null, has_school_id)
const SOFT_DELETE_TABLES: &[(&str, Option<&str>)] = &[
    ("teachers", Some("name")),
    ("school_staff", Some("name")),
    ("parents", Some("name")),
    ("classes", Some("name")),
    ("announcements", Some("title")),
    ("assignments", Some("title")),
    ("exams", Some("title")),
];

pub struct RecycleBinService {
    pub pool: PgPool,
}

impl RecycleBinService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
    }

    pub async fn get_deleted_entities(
        &self,
        entity_type: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        school_id: ObjectId,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<DeletedEntity>, AppError> {
        let school_id_str = school_id.to_hex();
        let mut deleted_entities: Vec<DeletedEntity> = Vec::new();

        for &(table_name, name_col) in SOFT_DELETE_TABLES {
            if let Some(ref filter_type) = entity_type {
                if filter_type != table_name { continue; }
            }

            let name_expr = name_col.map(|c| format!("{}", c)).unwrap_or_else(|| "NULL".to_string());

            let mut query_str = format!(
                "SELECT id, {} AS entity_name, deleted_at, school_id FROM {} WHERE deleted_at IS NOT NULL AND school_id = $1",
                name_expr, table_name
            );

            let mut param_idx = 2;

            if start_date.is_some() {
                query_str.push_str(&format!(" AND deleted_at >= ${}", param_idx));
                param_idx += 1;
            }
            if end_date.is_some() {
                query_str.push_str(&format!(" AND deleted_at <= ${}", param_idx));
            }

            let mut q = sqlx::query(&query_str).bind(&school_id_str);
            if let Some(sd) = start_date { q = q.bind(sd); }
            if let Some(ed) = end_date { q = q.bind(ed); }

            let rows = q.fetch_all(&self.pool).await.map_err(Self::db_error)?;

            for row in rows {
                let entity_id: String = row.try_get("id").unwrap_or_default();
                let entity_name: Option<String> = row.try_get("entity_name").ok().flatten();
                let del_at: DateTime<Utc> = row.try_get("deleted_at").unwrap_or_else(|_| Utc::now());
                let s_id: String = row.try_get("school_id").unwrap_or_default();

                deleted_entities.push(DeletedEntity {
                    entity_type: table_name.to_string(),
                    entity_id,
                    entity_name,
                    deleted_at: del_at,
                    deleted_by: String::new(),
                    deleted_by_name: None,
                    school_id: s_id,
                });
            }
        }

        deleted_entities.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));

        let total = deleted_entities.len() as i64;
        let skip_val = skip.unwrap_or(0) as usize;
        let limit_val = limit.unwrap_or(20) as usize;

        let data: Vec<DeletedEntity> = deleted_entities.into_iter().skip(skip_val).take(limit_val).collect();
        let total_pages = (total as f64 / limit_val as f64).ceil() as i64;
        let current_page = (skip_val / limit_val) as i64 + 1;

        Ok(Paginated { data, total, total_pages, current_page })
    }

    fn resolve_table(entity_type: &str) -> Option<&'static str> {
        SOFT_DELETE_TABLES.iter().find(|(t, _)| *t == entity_type).map(|(t, _)| *t)
    }

    pub async fn restore_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
        user: &AuthUserDto,
        state: &crate::config::state::AppState,
    ) -> Result<(), AppError> {
        let table = Self::resolve_table(entity_type).ok_or_else(|| AppError { message: format!("Unknown entity type: {}", entity_type) })?;

        let deleted_at: Option<DateTime<Utc>> = sqlx::query_scalar(
            &format!("SELECT deleted_at FROM {} WHERE id = $1", table)
        )
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?
        .flatten();

        if deleted_at.is_none() {
            return Err(AppError { message: "Entity is not deleted or not found".to_string() });
        }

        sqlx::query(&format!("UPDATE {} SET deleted_at = NULL, updated_at = now() WHERE id = $1", table))
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let school_oid = ObjectId::new(); // fallback
        let entity_oid = ObjectId::parse_str(entity_id).unwrap_or_else(|_| ObjectId::new());
        let audit_service = AuditLogService::new(&state.pg.pool);
        audit_service.log_event(school_oid, user, &format!("{}.restore", entity_type), entity_type, entity_oid, None, None, None).await.ok();

        Ok(())
    }

    pub async fn permanently_delete(
        &self,
        entity_type: &str,
        entity_id: &str,
        user: &AuthUserDto,
        state: &crate::config::state::AppState,
    ) -> Result<(), AppError> {
        let table = Self::resolve_table(entity_type).ok_or_else(|| AppError { message: format!("Unknown entity type: {}", entity_type) })?;

        let deleted_at: Option<DateTime<Utc>> = sqlx::query_scalar(
            &format!("SELECT deleted_at FROM {} WHERE id = $1", table)
        )
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?
        .flatten();

        if deleted_at.is_none() {
            return Err(AppError { message: "Entity is not deleted or not found".to_string() });
        }

        sqlx::query(&format!("DELETE FROM {} WHERE id = $1", table))
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let school_oid = ObjectId::new();
        let entity_oid = ObjectId::parse_str(entity_id).unwrap_or_else(|_| ObjectId::new());
        let audit_service = AuditLogService::new(&state.pg.pool);
        audit_service.log_event(school_oid, user, &format!("{}.permanent_delete", entity_type), entity_type, entity_oid, None, None, None).await.ok();

        Ok(())
    }
}
