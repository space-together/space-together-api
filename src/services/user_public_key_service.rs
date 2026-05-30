use chrono::Utc;
use sqlx::{PgPool, Row};

use crate::{
    domain::user_public_key::{PublicKeyInfo, UserPublicKey},
    errors::AppError,
    utils::object_id::ObjectId,
};

pub struct UserPublicKeyService {
    pub pool: PgPool,
}

impl UserPublicKeyService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
    }

    fn new_id() -> String {
        ObjectId::new().to_hex()
    }

    fn parse_oid(raw: &str, field: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(raw).map_err(|e| AppError {
            message: format!("Invalid {} ObjectId-compatible ID: {}", field, e),
        })
    }

    fn row_to_key(row: sqlx::postgres::PgRow) -> Result<UserPublicKey, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let user_id_str: String = row.try_get("user_id").map_err(Self::db_error)?;
        Ok(UserPublicKey {
            id: Some(Self::parse_oid(&id, "id")?),
            user_id: Self::parse_oid(&user_id_str, "user_id")?,
            public_key: row.try_get("public_key").map_err(Self::db_error)?,
            key_algorithm: row.try_get("key_algorithm").map_err(Self::db_error)?,
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        })
    }

    pub async fn upsert_public_key(
        &self,
        user_id: ObjectId,
        public_key: String,
        key_algorithm: String,
    ) -> Result<UserPublicKey, AppError> {
        if !public_key.contains("BEGIN PUBLIC KEY") || !public_key.contains("END PUBLIC KEY") {
            return Err(AppError {
                message: "Invalid public key format. Must be PEM format.".to_string(),
            });
        }

        let id = Self::new_id();
        let user_id_str = user_id.to_hex();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO user_public_keys (id, user_id, public_key, key_algorithm, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (user_id) DO UPDATE
               SET public_key = $3, key_algorithm = $4, updated_at = $6"#,
        )
        .bind(&id)
        .bind(&user_id_str)
        .bind(&public_key)
        .bind(&key_algorithm)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.get_public_key(user_id).await
    }

    pub async fn get_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError> {
        let user_id_str = user_id.to_hex();

        let row = sqlx::query(
            "SELECT id, user_id, public_key, key_algorithm, created_at, updated_at \
             FROM user_public_keys WHERE user_id = $1",
        )
        .bind(&user_id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        match row {
            Some(r) => Self::row_to_key(r),
            None => Err(AppError { message: format!("Public key not found for user: {}", user_id) }),
        }
    }

    pub async fn get_public_keys(
        &self,
        user_ids: Vec<ObjectId>,
    ) -> Result<Vec<PublicKeyInfo>, AppError> {
        let (keys, missing) = self.get_public_keys_partial(user_ids).await?;
        if !missing.is_empty() {
            return Err(AppError {
                message: format!("Public key not found for users: {}", missing.join(", ")),
            });
        }
        Ok(keys)
    }

    pub async fn get_public_keys_partial(
        &self,
        user_ids: Vec<ObjectId>,
    ) -> Result<(Vec<PublicKeyInfo>, Vec<String>), AppError> {
        if user_ids.is_empty() {
            return Err(AppError { message: "user_ids parameter is required".to_string() });
        }
        if user_ids.len() > 50 {
            return Err(AppError { message: "Maximum 50 user IDs allowed per request".to_string() });
        }

        let strings: Vec<String> = user_ids.iter().map(|o| o.to_hex()).collect();

        let mut sql = sqlx::QueryBuilder::new(
            "SELECT id, user_id, public_key, key_algorithm, created_at, updated_at \
             FROM user_public_keys WHERE user_id IN (",
        );
        let mut sep = sql.separated(", ");
        for id in &strings { sep.push_bind(id); }
        sep.push_unseparated(")");

        let rows = sql.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let keys: Vec<UserPublicKey> = rows.into_iter().map(Self::row_to_key).collect::<Result<Vec<_>, _>>()?;

        let found_ids: Vec<String> = keys.iter().map(|k| k.user_id.to_hex()).collect();
        let missing: Vec<String> = strings.into_iter().filter(|id| !found_ids.contains(id)).collect();

        let public_keys = keys.into_iter().map(|k| PublicKeyInfo {
            user_id: k.user_id.to_hex(),
            public_key: k.public_key,
            key_algorithm: k.key_algorithm,
            created_at: k.created_at,
        }).collect();

        Ok((public_keys, missing))
    }

    pub async fn get_or_create_public_keys(
        &self,
        user_ids: Vec<ObjectId>,
    ) -> Result<Vec<PublicKeyInfo>, AppError> {
        if user_ids.is_empty() {
            return Err(AppError { message: "user_ids parameter is required".to_string() });
        }
        if user_ids.len() > 50 {
            return Err(AppError { message: "Maximum 50 user IDs allowed per request".to_string() });
        }

        let (mut public_keys, missing_ids) = self.get_public_keys_partial(user_ids.clone()).await?;

        for missing_id_str in missing_ids {
            let user_id = Self::parse_oid(&missing_id_str, "user_id")?;
            let created_key = self.try_create_public_key(user_id).await?;
            public_keys.push(PublicKeyInfo {
                user_id: created_key.user_id.to_hex(),
                public_key: created_key.public_key,
                key_algorithm: created_key.key_algorithm,
                created_at: created_key.created_at,
            });
        }

        Ok(public_keys)
    }

    async fn try_create_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError> {
        let public_key = crate::utils::crypto_utils::generate_rsa_public_key()?;
        self.upsert_public_key(user_id, public_key, "RSA-2048".to_string()).await
    }

    pub async fn delete_public_key(&self, user_id: ObjectId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM user_public_keys WHERE user_id = $1")
            .bind(user_id.to_hex())
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(())
    }
}
