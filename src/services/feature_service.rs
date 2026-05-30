use sqlx::PgPool;

use crate::{errors::AppError, models::id_model::IdType};

pub struct FeatureService {
    pub pool: PgPool,
}

impl FeatureService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
    }

    fn id_to_string(id: &IdType) -> Result<String, AppError> {
        Ok(IdType::to_object_id(id)?.to_hex())
    }

    pub async fn is_feature_enabled(
        &self,
        school_id: &IdType,
        feature_name: &str,
    ) -> Result<bool, AppError> {
        let school_id_str = Self::id_to_string(school_id)?;

        let row: Option<bool> = sqlx::query_scalar(
            "SELECT enabled FROM school_features WHERE school_id = $1 AND feature_name = $2",
        )
        .bind(&school_id_str)
        .bind(feature_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(row.unwrap_or(true))
    }

    pub async fn toggle_feature(
        &self,
        school_id: &IdType,
        feature_name: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let school_id_str = Self::id_to_string(school_id)?;

        sqlx::query(
            r#"INSERT INTO school_features (school_id, feature_name, enabled, updated_at)
               VALUES ($1, $2, $3, now())
               ON CONFLICT (school_id, feature_name) DO UPDATE SET enabled = $3, updated_at = now()"#,
        )
        .bind(&school_id_str)
        .bind(feature_name)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(())
    }
}
