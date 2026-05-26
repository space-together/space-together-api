use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::{Paginated, UserRole},
        conversation::{Conversation, ConversationKey, ConversationWithRelations},
    },
    errors::AppError,
    models::id_model::IdType,
    schema::common_schema::ActorRef,
    utils::object_id::ObjectId,
};

pub struct ConversationService {
    pub pool: PgPool,
}

impl ConversationService {
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

    fn conversation_from_row(row: &sqlx::postgres::PgRow) -> Result<Conversation, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Conversation {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            participants: Vec::new(),
            is_group: row.try_get("is_group").ok().unwrap_or(false),
            name: row.try_get("name").ok().flatten(),
            encryption_key_version: row.try_get("encryption_key_version").ok().unwrap_or(1),
            created_at: row.try_get("created_at").map_err(Self::db_error)?,
            updated_at: row.try_get("updated_at").map_err(Self::db_error)?,
        })
    }

    async fn hydrate_participants(&self, conversation: &mut Conversation) -> Result<(), AppError> {
        let Some(id) = conversation.id else {
            return Ok(());
        };

        let rows = sqlx::query(
            r#"
            SELECT COALESCE(actor_id, user_id, student_id) AS actor_id,
                   COALESCE(actor_role, role) AS actor_role
            FROM conversation_participants
            WHERE conversation_id = $1 AND left_at IS NULL
            ORDER BY joined_at ASC, id ASC
            "#,
        )
        .bind(id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut participants = Vec::with_capacity(rows.len());
        for row in rows {
            let actor_id: String = row.try_get("actor_id").map_err(Self::db_error)?;
            participants.push(ActorRef {
                id: Self::parse_oid(&actor_id, "participant_id")?,
                role: Self::role_from_string(row.try_get("actor_role").ok().flatten()),
            });
        }
        conversation.participants = participants;
        Ok(())
    }

    async fn conversation_with_relations(
        &self,
        mut conversation: Conversation,
    ) -> Result<ConversationWithRelations, AppError> {
        self.hydrate_participants(&mut conversation).await?;
        Ok(ConversationWithRelations {
            conversation,
            participants_users: Vec::new(),
        })
    }

    pub async fn create(&self, mut dto: Conversation) -> Result<Conversation, AppError> {
        self.ensure_indexes().await?;

        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        let school_id = dto.school_id.map(|id| id.to_hex());
        let now = Utc::now();
        let created_at = dto.created_at;
        let updated_at = now;

        let mut tx = self.pool.begin().await.map_err(Self::db_error)?;
        sqlx::query(
            r#"
            INSERT INTO conversations (
              id, school_id, conversation_type, title, is_group, name,
              encryption_key_version, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(if dto.is_group { "GROUP" } else { "DIRECT" })
        .bind(&dto.name)
        .bind(dto.is_group)
        .bind(&dto.name)
        .bind(dto.encryption_key_version)
        .bind(created_at)
        .bind(updated_at)
        .execute(&mut *tx)
        .await
        .map_err(Self::db_error)?;

        for participant in &dto.participants {
            sqlx::query(
                r#"
                INSERT INTO conversation_participants (
                  id, conversation_id, school_id, actor_id, actor_role, role, joined_at
                )
                VALUES ($1, $2, $3, $4, $5, $5, $6)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(Self::new_id())
            .bind(&id)
            .bind(&school_id)
            .bind(participant.id.to_hex())
            .bind(Self::role_to_string(&participant.role))
            .bind(created_at)
            .execute(&mut *tx)
            .await
            .map_err(Self::db_error)?;
        }

        tx.commit().await.map_err(Self::db_error)?;
        dto.id = Some(Self::parse_oid(&id, "id")?);
        dto.created_at = created_at;
        dto.updated_at = updated_at;
        Ok(dto)
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        participant_id: Option<ObjectId>,
    ) -> Result<Conversation, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT c.id, c.school_id, c.is_group, c.name, c.encryption_key_version,
                   c.created_at, c.updated_at
            FROM conversations c
            WHERE c.deleted_at IS NULL
            "#,
        );

        if let Some(id) = id {
            query
                .push(" AND c.id = ")
                .push_bind(Self::id_to_string(id)?);
        }

        if let Some(participant_id) = participant_id {
            query.push(
                r#"
                AND EXISTS (
                  SELECT 1 FROM conversation_participants cp
                  WHERE cp.conversation_id = c.id
                    AND cp.left_at IS NULL
                    AND COALESCE(cp.actor_id, cp.user_id, cp.student_id) =
                "#,
            );
            query.push_bind(participant_id.to_hex()).push(")");
        }

        query.push(" ORDER BY c.updated_at DESC LIMIT 1");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .ok_or(AppError {
                message: "Conversation not found".into(),
            })?;

        let mut conversation = Self::conversation_from_row(&row)?;
        self.hydrate_participants(&mut conversation).await?;
        Ok(conversation)
    }

    pub async fn find_direct_conversation(
        &self,
        participants: &[ActorRef],
        school_id: Option<ObjectId>,
    ) -> Result<Option<Conversation>, AppError> {
        let participant_ids: Vec<String> = participants.iter().map(|p| p.id.to_hex()).collect();

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT c.id, c.school_id, c.is_group, c.name, c.encryption_key_version,
                   c.created_at, c.updated_at
            FROM conversations c
            WHERE c.deleted_at IS NULL
              AND c.is_group = false
              AND (
            "#,
        );

        match school_id {
            Some(id) => {
                query.push("c.school_id = ").push_bind(id.to_hex());
            }
            None => {
                query.push("c.school_id IS NULL");
            }
        }

        query.push(
            r#")
              AND (
                SELECT count(*)
                FROM conversation_participants cp
                WHERE cp.conversation_id = c.id AND cp.left_at IS NULL
              ) = "#,
        );
        query.push_bind(participant_ids.len() as i64);
        query.push(
            r#"
              AND (
                SELECT count(*)
                FROM conversation_participants cp
                WHERE cp.conversation_id = c.id
                  AND cp.left_at IS NULL
                  AND COALESCE(cp.actor_id, cp.user_id, cp.student_id) IN (
            "#,
        );
        let mut separated = query.separated(", ");
        for id in &participant_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        query.push(") = ").push_bind(participant_ids.len() as i64);
        query.push(" ORDER BY c.updated_at DESC LIMIT 1");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => {
                let mut conversation = Self::conversation_from_row(&row)?;
                self.hydrate_participants(&mut conversation).await?;
                Ok(Some(conversation))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_with_relations(
        &self,
        limit: Option<i64>,
        skip: Option<i64>,
        participant_id: ObjectId,
        school_id: Option<ObjectId>,
    ) -> Result<Paginated<ConversationWithRelations>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT count(DISTINCT c.id)::BIGINT
            FROM conversations c
            JOIN conversation_participants cp ON cp.conversation_id = c.id
            WHERE c.deleted_at IS NULL
              AND cp.left_at IS NULL
              AND COALESCE(cp.actor_id, cp.user_id, cp.student_id) =
            "#,
        );
        count_query.push_bind(participant_id.to_hex());
        Self::push_school_filter(&mut count_query, school_id);

        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT DISTINCT c.id, c.school_id, c.is_group, c.name, c.encryption_key_version,
                   c.created_at, c.updated_at
            FROM conversations c
            JOIN conversation_participants cp ON cp.conversation_id = c.id
            WHERE c.deleted_at IS NULL
              AND cp.left_at IS NULL
              AND COALESCE(cp.actor_id, cp.user_id, cp.student_id) =
            "#,
        );
        query.push_bind(participant_id.to_hex());
        Self::push_school_filter(&mut query, school_id);
        query
            .push(" ORDER BY c.updated_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(
                self.conversation_with_relations(Self::conversation_from_row(&row)?)
                    .await?,
            );
        }

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

    fn push_school_filter(query: &mut QueryBuilder<'_, Postgres>, school_id: Option<ObjectId>) {
        match school_id {
            Some(school_id) => {
                query
                    .push(" AND c.school_id = ")
                    .push_bind(school_id.to_hex());
            }
            None => {
                query.push(" AND c.school_id IS NULL");
            }
        }
    }

    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
        participant_id: ObjectId,
    ) -> Result<ConversationWithRelations, AppError> {
        let conversation = self.find_one(Some(id), Some(participant_id)).await?;
        self.conversation_with_relations(conversation).await
    }

    pub async fn store_conversation_key(
        &self,
        key: ConversationKey,
    ) -> Result<ConversationKey, AppError> {
        let id = key.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        sqlx::query(
            r#"
            INSERT INTO conversation_keys (
              id, conversation_id, actor_id, actor_role, user_role,
              encrypted_key, encrypted_key_for_user, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $4, $5, $5, $6, $6)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&id)
        .bind(key.conversation_id.to_hex())
        .bind(key.user_id.to_hex())
        .bind(Self::role_to_string(&key.user_role))
        .bind(&key.encrypted_key_for_user)
        .bind(key.created_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.get_conversation_key(key.conversation_id, key.user_id)
            .await
    }

    pub async fn get_conversation_key(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<ConversationKey, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, conversation_id, COALESCE(actor_id, user_id) AS user_id,
                   COALESCE(actor_role, user_role) AS user_role,
                   COALESCE(encrypted_key_for_user, encrypted_key) AS encrypted_key_for_user,
                   created_at
            FROM conversation_keys
            WHERE conversation_id = $1 AND COALESCE(actor_id, user_id) = $2
            LIMIT 1
            "#,
        )
        .bind(conversation_id.to_hex())
        .bind(user_id.to_hex())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?
        .ok_or(AppError {
            message: "Conversation key not found".into(),
        })?;

        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let conversation_id: String = row.try_get("conversation_id").map_err(Self::db_error)?;
        let user_id: String = row.try_get("user_id").map_err(Self::db_error)?;
        Ok(ConversationKey {
            id: Some(Self::parse_oid(&id, "id")?),
            conversation_id: Self::parse_oid(&conversation_id, "conversation_id")?,
            user_id: Self::parse_oid(&user_id, "user_id")?,
            user_role: Self::role_from_string(row.try_get("user_role").ok().flatten()),
            encrypted_key_for_user: row
                .try_get("encrypted_key_for_user")
                .map_err(Self::db_error)?,
            created_at: row
                .try_get::<DateTime<Utc>, _>("created_at")
                .map_err(Self::db_error)?,
        })
    }

    pub async fn is_participant(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)::BIGINT
            FROM conversation_participants
            WHERE conversation_id = $1
              AND left_at IS NULL
              AND COALESCE(actor_id, user_id, student_id) = $2
            "#,
        )
        .bind(conversation_id.to_hex())
        .bind(user_id.to_hex())
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(count > 0)
    }
}
