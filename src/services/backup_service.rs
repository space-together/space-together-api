use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};
use std::path::PathBuf;
use std::process::Command;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        backup::{BackupStatus, BackupType, SchoolBackup, SchoolBackupWithRelations},
        common_details::Paginated,
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::CountDoc},
    services::audit_log_service::AuditLogService,
    utils::object_id::ObjectId,
};

pub struct BackupService {
    pub pool: PgPool,
}

impl BackupService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
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

    fn backup_type_to_string(t: &BackupType) -> String {
        match t {
            BackupType::Manual => "Manual",
            BackupType::Automated => "Automated",
        }.to_string()
    }

    fn backup_type_from_string(raw: &str) -> BackupType {
        match raw {
            "Automated" => BackupType::Automated,
            _ => BackupType::Manual,
        }
    }

    fn backup_status_to_string(s: &BackupStatus) -> String {
        match s {
            BackupStatus::InProgress => "InProgress",
            BackupStatus::Completed => "Completed",
            BackupStatus::Failed => "Failed",
        }.to_string()
    }

    fn backup_status_from_string(raw: &str) -> BackupStatus {
        match raw {
            "Completed" => BackupStatus::Completed,
            "Failed" => BackupStatus::Failed,
            _ => BackupStatus::InProgress,
        }
    }

    fn row_to_backup(row: PgRow) -> Result<SchoolBackup, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let school_id_str: Option<String> = row.try_get("school_id").ok().flatten();
        let created_by_str: Option<String> = row.try_get("created_by").ok().flatten();
        let type_str: String = row.try_get("backup_type").unwrap_or_else(|_| "Manual".to_string());
        let status_str: String = row.try_get("status").unwrap_or_else(|_| "InProgress".to_string());

        Ok(SchoolBackup {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: school_id_str.as_deref().map(|s| Self::parse_oid(s, "school_id")).transpose()?,
            backup_name: row.try_get("backup_name").map_err(Self::db_error)?,
            backup_type: Self::backup_type_from_string(&type_str),
            file_path: row.try_get("file_path").map_err(Self::db_error)?,
            size_bytes: row.try_get("size_bytes").unwrap_or(0),
            status: Self::backup_status_from_string(&status_str),
            created_by: created_by_str.as_deref().map(|s| Self::parse_oid(s, "created_by")).transpose()?,
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            completed_at: row.try_get("completed_at").ok().flatten(),
            error_message: row.try_get("error_message").ok().flatten(),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, school_id, backup_name, backup_type, file_path, size_bytes, status, \
         created_by, created_at, completed_at, error_message FROM school_backups WHERE 1=1"
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(backup_name) LIKE ").push_bind(like)
            .push(" OR lower(status) LIKE ").push_bind(like)
            .push(" OR lower(backup_type) LIKE ").push_bind(like)
            .push(" OR lower(id) LIKE ").push_bind(like)
            .push(")");
    }

    pub async fn create_manual_backup(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        state: &AppState,
    ) -> Result<SchoolBackup, AppError> {
        let backup_name = format!("manual_backup_{}_{}", school_id.to_hex(), Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = format!("backups/{}.archive", backup_name);
        let id = Self::new_id();
        let now = Utc::now();
        let created_by = ObjectId::parse_str(&user.id).map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

        sqlx::query(
            r#"INSERT INTO school_backups (id, school_id, backup_name, backup_type, file_path, size_bytes, status, created_by, created_at)
               VALUES ($1,$2,$3,$4,$5,0,$6,$7,$8)"#,
        )
        .bind(&id)
        .bind(school_id.to_hex())
        .bind(&backup_name)
        .bind(Self::backup_type_to_string(&BackupType::Manual))
        .bind(&file_path)
        .bind(Self::backup_status_to_string(&BackupStatus::InProgress))
        .bind(created_by.to_hex())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let created_backup = self.find_one(Some(&IdType::from_string(id.clone())), None).await?;

        let audit_service = AuditLogService::new(&state.pg.pool);
        audit_service.log_event(school_id, user, "backup.manual.create", "backup", created_by, None, None, None).await.ok();

        let pool_clone = self.pool.clone();
        let file_path_clone = file_path.clone();
        let db_name = format!("school_{}", school_id.to_hex());
        let backup_id = id.clone();

        actix_rt::spawn(async move {
            let result = Self::perform_backup(&db_name, &file_path_clone).await;
            match result {
                Ok(size) => {
                    let _ = sqlx::query("UPDATE school_backups SET status = 'Completed', size_bytes = $1, completed_at = now() WHERE id = $2")
                        .bind(size).bind(&backup_id).execute(&pool_clone).await;
                }
                Err(e) => {
                    let _ = sqlx::query("UPDATE school_backups SET status = 'Failed', error_message = $1, completed_at = now() WHERE id = $2")
                        .bind(e.message).bind(&backup_id).execute(&pool_clone).await;
                }
            }
        });

        Ok(created_backup)
    }

    async fn perform_backup(db_name: &str, file_path: &str) -> Result<i64, AppError> {
        let backup_dir = PathBuf::from("backups");
        std::fs::create_dir_all(&backup_dir).map_err(|e| AppError { message: format!("Failed to create backup directory: {}", e) })?;

        let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let output = Command::new("mongodump")
            .arg("--uri").arg(&mongo_uri)
            .arg("--db").arg(db_name)
            .arg("--archive").arg(file_path)
            .arg("--gzip")
            .output()
            .map_err(|e| AppError { message: format!("Failed to execute mongodump: {}", e) })?;

        if !output.status.success() {
            return Err(AppError { message: format!("Backup failed: {}", String::from_utf8_lossy(&output.stderr)) });
        }

        let metadata = std::fs::metadata(file_path).map_err(|e| AppError { message: format!("Failed to get backup file size: {}", e) })?;
        Ok(metadata.len() as i64)
    }

    pub async fn restore_backup(
        &self,
        backup_id: &IdType,
        user: &AuthUserDto,
        state: &AppState,
    ) -> Result<SchoolBackup, AppError> {
        let backup = self.find_one(Some(backup_id), None).await?;

        if !matches!(backup.status, BackupStatus::Completed) {
            return Err(AppError { message: "Cannot restore: backup is not completed".to_string() });
        }

        if !std::path::Path::new(&backup.file_path).exists() {
            return Err(AppError { message: "Backup file not found".to_string() });
        }

        let school_id = backup.school_id.ok_or(AppError { message: "Backup has no school_id".to_string() })?;

        let ongoing: i64 = sqlx::query_scalar("SELECT count(*) FROM school_backups WHERE school_id = $1 AND status = 'InProgress'")
            .bind(school_id.to_hex())
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        if ongoing > 0 {
            return Err(AppError { message: "Another restore is already in progress for this school".to_string() });
        }

        let restore_id = Self::new_id();
        let created_by = ObjectId::parse_str(&user.id).map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

        sqlx::query(
            r#"INSERT INTO school_backups (id, school_id, backup_name, backup_type, file_path, size_bytes, status, created_by, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,now())"#,
        )
        .bind(&restore_id)
        .bind(school_id.to_hex())
        .bind(format!("restore_{}", backup.backup_name))
        .bind(Self::backup_type_to_string(&BackupType::Manual))
        .bind(&backup.file_path)
        .bind(backup.size_bytes)
        .bind(Self::backup_status_to_string(&BackupStatus::InProgress))
        .bind(created_by.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let restore_doc = self.find_one(Some(&IdType::from_string(restore_id.clone())), None).await?;

        let audit_service = AuditLogService::new(&state.pg.pool);
        audit_service.log_event(school_id, user, "backup.restore", "backup", backup_id.to_object_id()?, None, None, None).await.ok();

        let pool_clone = self.pool.clone();
        let file_path = backup.file_path.clone();
        let db_name = format!("school_{}", school_id.to_hex());

        actix_rt::spawn(async move {
            let result = Self::perform_restore(&db_name, &file_path).await;
            match result {
                Ok(_) => {
                    let _ = sqlx::query("UPDATE school_backups SET status = 'Completed', completed_at = now() WHERE id = $1")
                        .bind(&restore_id).execute(&pool_clone).await;
                }
                Err(e) => {
                    let _ = sqlx::query("UPDATE school_backups SET status = 'Failed', error_message = $1, completed_at = now() WHERE id = $2")
                        .bind(e.message).bind(&restore_id).execute(&pool_clone).await;
                }
            }
        });

        Ok(restore_doc)
    }

    async fn perform_restore(db_name: &str, file_path: &str) -> Result<(), AppError> {
        let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let output = Command::new("mongorestore")
            .arg("--uri").arg(&mongo_uri)
            .arg("--db").arg(db_name)
            .arg("--archive").arg(file_path)
            .arg("--gzip")
            .arg("--drop")
            .output()
            .map_err(|e| AppError { message: format!("Failed to execute mongorestore: {}", e) })?;

        if !output.status.success() {
            return Err(AppError { message: format!("Restore failed: {}", String::from_utf8_lossy(&output.stderr)) });
        }
        Ok(())
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        school_id: Option<&str>,
    ) -> Result<SchoolBackup, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id { sql.push(" AND id = ").push_bind(Self::id_to_string(id)?); }
        if let Some(sid) = school_id { sql.push(" AND school_id = ").push_bind(sid); }
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_backup(r),
            None => Err(AppError { message: "Backup not found".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        school_id: Option<&str>,
    ) -> Result<Paginated<SchoolBackup>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter.as_ref().filter(|v| !v.trim().is_empty()).map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM school_backups WHERE 1=1");
        if let Some(sid) = school_id { cq.push(" AND school_id = ").push_bind(sid); }
        if let Some(ref l) = like { Self::push_search(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(sid) = school_id { dq.push(" AND school_id = ").push_bind(sid); }
        if let Some(ref l) = like { Self::push_search(&mut dq, l); }
        dq.push(" ORDER BY created_at DESC LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_backup).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        school_id: Option<&str>,
    ) -> Result<Paginated<SchoolBackupWithRelations>, AppError> {
        // For now return backups without extra relations (creator user)
        let result = self.get_all(filter, limit, skip, school_id).await?;
        let data = result.data.into_iter().map(|backup| SchoolBackupWithRelations { backup, school: None, creator: None }).collect();
        Ok(Paginated { data, total: result.total, total_pages: result.total_pages, current_page: result.current_page })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        school_id: Option<&str>,
    ) -> Result<SchoolBackupWithRelations, AppError> {
        let backup = self.find_one(id, school_id).await?;
        Ok(SchoolBackupWithRelations { backup, school: None, creator: None })
    }

    pub async fn delete(&self, id: &IdType) -> Result<SchoolBackup, AppError> {
        let backup = self.find_one(Some(id), None).await?;
        if std::path::Path::new(&backup.file_path).exists() {
            std::fs::remove_file(&backup.file_path).ok();
        }
        sqlx::query("DELETE FROM school_backups WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(backup)
    }

    pub async fn count_backups(
        &self,
        filter: Option<String>,
        school_id: Option<&str>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), school_id).await?.total;
        Ok(CountDoc { count: total as u64 })
    }
}
