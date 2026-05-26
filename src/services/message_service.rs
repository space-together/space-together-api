use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::{
    domain::{
        common_details::UserRole,
        message::{Message, MessageType, MessageWithRelations},
    },
    errors::AppError,
    models::id_model::IdType,
    schema::common_schema::ActorRef,
    utils::object_id::ObjectId,
};

pub struct MessageService {
    pub pool: PgPool,
    recent_message_ids: Arc<RwLock<HashSet<String>>>,
}

impl MessageService {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            pool: pool.clone(),
            recent_message_ids: Arc::new(RwLock::new(HashSet::new())),
        }
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

    fn message_type_to_string(message_type: &MessageType) -> &'static str {
        match message_type {
            MessageType::TEXT => "TEXT",
            MessageType::FILE => "FILE",
            MessageType::SYSTEM => "SYSTEM",
        }
    }

    fn message_type_from_string(raw: Option<String>) -> MessageType {
        match raw
            .unwrap_or_else(|| "TEXT".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "FILE" => MessageType::FILE,
            "SYSTEM" => MessageType::SYSTEM,
            _ => MessageType::TEXT,
        }
    }

    fn message_from_row(row: &sqlx::postgres::PgRow) -> Result<Message, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let conversation_id: String = row.try_get("conversation_id").map_err(Self::db_error)?;
        let sender_id: String = row.try_get("sender_actor_id").map_err(Self::db_error)?;

        Ok(Message {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            conversation_id: Self::parse_oid(&conversation_id, "conversation_id")?,
            sender: ActorRef {
                id: Self::parse_oid(&sender_id, "sender_id")?,
                role: Self::role_from_string(row.try_get("sender_role").ok().flatten()),
            },
            encrypted_payload: row
                .try_get::<Option<String>, _>("encrypted_payload")
                .ok()
                .flatten()
                .unwrap_or_default(),
            nonce: row
                .try_get::<Option<String>, _>("nonce")
                .ok()
                .flatten()
                .unwrap_or_default(),
            key_version: row.try_get("key_version").ok().unwrap_or(1),
            message_type: Self::message_type_from_string(
                row.try_get("message_type").ok().flatten(),
            ),
            file_url: row.try_get("file_url").ok().flatten(),
            file_public_id: row.try_get("file_public_id").ok().flatten(),
            read_by: Vec::new(),
            client_message_id: row
                .try_get::<Option<String>, _>("client_message_id")
                .ok()
                .flatten()
                .unwrap_or_default(),
            deleted_at: row.try_get("deleted_at").ok().flatten(),
            created_at: row
                .try_get::<DateTime<Utc>, _>("created_at")
                .map_err(Self::db_error)?,
        })
    }

    async fn hydrate_read_receipts(&self, message: &mut Message) -> Result<(), AppError> {
        let Some(id) = message.id else {
            return Ok(());
        };

        let rows = sqlx::query(
            r#"
            SELECT actor_id, actor_role
            FROM message_read_receipts
            WHERE message_id = $1
            ORDER BY read_at ASC
            "#,
        )
        .bind(id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let _ = rows;
        message.read_by = Vec::new();
        Ok(())
    }

    async fn with_relations(&self, mut message: Message) -> Result<MessageWithRelations, AppError> {
        self.hydrate_read_receipts(&mut message).await?;
        Ok(MessageWithRelations {
            message,
            sender_user: None,
        })
    }

    pub async fn create(&self, dto: Message) -> Result<Message, AppError> {
        self.ensure_indexes().await?;

        {
            let mut ids = self.recent_message_ids.write().await;
            if ids.contains(&dto.client_message_id) {
                return Err(AppError {
                    message: "Duplicate message detected".to_string(),
                });
            }
            ids.insert(dto.client_message_id.clone());

            if ids.len() > 10000 {
                ids.clear();
            }
        }

        if dto.encrypted_payload.len() > 100 * 1024 {
            return Err(AppError {
                message: "Encrypted payload too large".to_string(),
            });
        }

        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        sqlx::query(
            r#"
            INSERT INTO messages (
              id, conversation_id, school_id, sender_actor_id, sender_role,
              encrypted_payload, body, nonce, key_version, message_type,
              file_url, file_public_id, client_message_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $6, $7, $8, $9, $10, $11, $12, $13, $13)
            "#,
        )
        .bind(&id)
        .bind(dto.conversation_id.to_hex())
        .bind(dto.school_id.map(|id| id.to_hex()))
        .bind(dto.sender.id.to_hex())
        .bind(Self::role_to_string(&dto.sender.role))
        .bind(&dto.encrypted_payload)
        .bind(&dto.nonce)
        .bind(dto.key_version)
        .bind(Self::message_type_to_string(&dto.message_type))
        .bind(&dto.file_url)
        .bind(&dto.file_public_id)
        .bind(&dto.client_message_id)
        .bind(dto.created_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if dto.message_type == MessageType::FILE {
            if let Some(file_url) = &dto.file_url {
                sqlx::query(
                    r#"
                    INSERT INTO message_attachments (id, message_id, url, public_id)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(Self::new_id())
                .bind(&id)
                .bind(file_url)
                .bind(&dto.file_public_id)
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
            }
        }

        self.find_one(&IdType::String(id)).await
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Message, AppError> {
        let row = sqlx::query(Self::select_sql())
            .bind(Self::id_to_string(id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .ok_or(AppError {
                message: "Message not found".into(),
            })?;

        let mut message = Self::message_from_row(&row)?;
        self.hydrate_read_receipts(&mut message).await?;
        Ok(message)
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, conversation_id, school_id,
               COALESCE(sender_actor_id, sender_user_id, sender_student_id) AS sender_actor_id,
               sender_role,
               COALESCE(encrypted_payload, body, '') AS encrypted_payload,
               COALESCE(nonce, '') AS nonce,
               COALESCE(key_version, 1) AS key_version,
               COALESCE(message_type, 'TEXT') AS message_type,
               file_url, file_public_id, client_message_id, deleted_at, created_at
        FROM messages
        WHERE id = $1 AND deleted_at IS NULL
        "#
    }

    pub async fn get_conversation_messages(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let (messages, total) = self
            .get_messages_for_conversation(conversation_id, page, limit, None)
            .await?;
        Ok((messages, total))
    }

    pub async fn get_conversation_files(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        self.get_messages_for_conversation(conversation_id, page, limit, Some(MessageType::FILE))
            .await
    }

    async fn get_messages_for_conversation(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
        message_type: Option<MessageType>,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let page = page.max(1);
        let limit = limit.max(1);
        let skip = (page - 1) * limit;

        let mut count_sql = String::from(
            "SELECT count(*)::BIGINT FROM messages WHERE conversation_id = $1 AND deleted_at IS NULL",
        );
        if message_type.is_some() {
            count_sql.push_str(" AND message_type = $2");
        }
        let mut count_query =
            sqlx::query_scalar::<_, i64>(&count_sql).bind(conversation_id.to_hex());
        if let Some(message_type) = &message_type {
            count_query = count_query.bind(Self::message_type_to_string(message_type));
        }
        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data_sql = String::from(
            r#"
            SELECT id, conversation_id, school_id,
                   COALESCE(sender_actor_id, sender_user_id, sender_student_id) AS sender_actor_id,
                   sender_role,
                   COALESCE(encrypted_payload, body, '') AS encrypted_payload,
                   COALESCE(nonce, '') AS nonce,
                   COALESCE(key_version, 1) AS key_version,
                   COALESCE(message_type, 'TEXT') AS message_type,
                   file_url, file_public_id, client_message_id, deleted_at, created_at
            FROM messages
            WHERE conversation_id = $1 AND deleted_at IS NULL
            "#,
        );
        if message_type.is_some() {
            data_sql.push_str(" AND message_type = $2");
        }
        if message_type.is_some() {
            data_sql.push_str(" ORDER BY created_at DESC LIMIT $3 OFFSET $4");
        } else {
            data_sql.push_str(" ORDER BY created_at DESC LIMIT $2 OFFSET $3");
        }

        let mut data_query = sqlx::query(&data_sql).bind(conversation_id.to_hex());
        if let Some(message_type) = &message_type {
            data_query = data_query.bind(Self::message_type_to_string(message_type));
        }
        let rows = data_query
            .bind(limit)
            .bind(skip)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            let mut message = Self::message_from_row(&row)?;
            self.hydrate_read_receipts(&mut message).await?;
            messages.push(message);
        }

        Ok((messages, total))
    }

    pub async fn soft_delete(&self, id: &IdType) -> Result<Message, AppError> {
        let message = self.find_one(id).await?;
        sqlx::query("UPDATE messages SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(message)
    }

    pub async fn get_conversation_messages_with_relations(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<MessageWithRelations>, i64), AppError> {
        let (messages, total) = self
            .get_conversation_messages(conversation_id, page, limit)
            .await?;

        let mut data = Vec::with_capacity(messages.len());
        for message in messages {
            data.push(self.with_relations(message).await?);
        }
        Ok((data, total))
    }

    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<MessageWithRelations, AppError> {
        let message = self.find_one(id).await?;
        self.with_relations(message).await
    }
}
