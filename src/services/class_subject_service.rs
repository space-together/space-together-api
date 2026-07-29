use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        class_subject::{ClassSubject, ClassSubjectPartial, ClassSubjectWithRelations},
        common_details::{Paginated, SubjectCategory},
        template_subject::TemplateTopic,
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    services::{
        class_service::ClassService, school_service::SchoolService, teacher_service::TeacherService,
    },
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct ClassSubjectQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub class_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl ClassSubjectQuery {
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
            class_id: query.class_id.clone(),
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

pub struct ClassSubjectService {
    pub pool: PgPool,
}

impl ClassSubjectService {
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

    fn enum_to_string<T: Serialize>(value: &T) -> Option<String> {
        match serde_json::to_value(value).ok()? {
            serde_json::Value::String(value) => Some(value),
            other => Some(other.to_string()),
        }
    }

    fn enum_from_string<T: DeserializeOwned>(value: Option<String>) -> Option<T> {
        value.and_then(|raw| serde_json::from_value(serde_json::Value::String(raw)).ok())
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, teacher_id, school_id, class_id, main_subject_id, name, code, description,
               category, estimated_hours, credits, created_by, disable, created_at, updated_at
        FROM class_subjects
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a ClassSubjectQuery>,
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
        if let Some(class_id) = &query.class_id {
            parse_object_id_value(class_id)?;
            sql.push(" AND class_id = ").push_bind(class_id);
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
                "class_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND class_id = ").push_bind(value);
                }
                "teacher_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND teacher_id = ").push_bind(value);
                }
                "main_subject_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND main_subject_id = ").push_bind(value);
                }
                "created_by" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND created_by = ").push_bind(value);
                }
                "name" => {
                    sql.push(" AND lower(name) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "code" => {
                    sql.push(" AND lower(code) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "disable" => {
                    let parsed = value.parse::<bool>().map_err(|_| AppError {
                        message: "Invalid disable filter value".into(),
                    })?;
                    sql.push(" AND disable = ").push_bind(parsed);
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported class subject filter field: {}", field),
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
            .push(" OR lower(coalesce(category, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    async fn fetch_topics(&self, subject_id: &str) -> Result<Option<Vec<TemplateTopic>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, parent_topic_id, order_key, title, description, estimated_hours, credits
            FROM class_subject_topics
            WHERE class_subject_id = $1
            ORDER BY parent_topic_id NULLS FIRST, position ASC, created_at ASC
            "#,
        )
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut roots = Vec::new();
        for row in rows {
            if row
                .try_get::<Option<String>, _>("parent_topic_id")
                .ok()
                .flatten()
                .is_none()
            {
                roots.push(TemplateTopic {
                    order: row.try_get("order_key").map_err(Self::db_error)?,
                    title: row.try_get("title").map_err(Self::db_error)?,
                    description: row.try_get("description").ok().flatten(),
                    estimated_hours: row.try_get("estimated_hours").ok().flatten(),
                    credits: row.try_get("credits").ok().flatten(),
                    subtopics: None,
                });
            }
        }

        Ok((!roots.is_empty()).then_some(roots))
    }

    async fn sync_topics(
        &self,
        subject_id: &str,
        topics: &[TemplateTopic],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM class_subject_topics WHERE class_subject_id = $1")
            .bind(subject_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, topic) in topics.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO class_subject_topics (
                  id, class_subject_id, order_key, title, description, estimated_hours, credits, position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(Self::new_id())
            .bind(subject_id)
            .bind(&topic.order)
            .bind(&topic.title)
            .bind(&topic.description)
            .bind(topic.estimated_hours)
            .bind(topic.credits)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }
        Ok(())
    }

    async fn subject_from_row(&self, row: PgRow) -> Result<ClassSubject, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(ClassSubject {
            id: Some(Self::parse_oid(&id, "id")?),
            teacher_id: Self::parse_oid_opt(
                row.try_get("teacher_id").ok().flatten(),
                "teacher_id",
            )?,
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            main_subject_id: Self::parse_oid_opt(
                row.try_get("main_subject_id").ok().flatten(),
                "main_subject_id",
            )?,
            name: row.try_get("name").map_err(Self::db_error)?,
            code: row
                .try_get::<Option<String>, _>("code")
                .ok()
                .flatten()
                .unwrap_or_default(),
            description: row.try_get("description").ok().flatten(),
            category: Self::enum_from_string::<SubjectCategory>(
                row.try_get("category").ok().flatten(),
            )
            .unwrap_or_else(|| SubjectCategory::Other("Other".into())),
            estimated_hours: row.try_get("estimated_hours").ok().unwrap_or_default(),
            credits: row.try_get("credits").ok().flatten(),
            topics: self.fetch_topics(&id).await?,
            created_by: Self::parse_oid_opt(
                row.try_get("created_by").ok().flatten(),
                "created_by",
            )?,
            disable: row.try_get("disable").ok().flatten(),
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
        })
    }

    async fn code_exists(
        &self,
        code: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<ClassSubject>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(Self::select_sql());
        query
            .push(" AND lower(code) = lower(")
            .push_bind(code)
            .push(")");
        if let Some(school_id) = school_id {
            query.push(" AND school_id = ").push_bind(school_id);
        }
        if let Some(except_id) = except_id {
            query.push(" AND id <> ").push_bind(except_id);
        }
        query.push(" ORDER BY updated_at DESC LIMIT 1");
        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.subject_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(&self, dto: ClassSubject) -> Result<ClassSubject, AppError> {
        self.ensure_indexes().await?;
        let school_id = dto
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let class_id = dto
            .class_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "class_id is required".into(),
            })?;

        if let Some(existing) = self.code_exists(&dto.code, Some(&school_id), None).await? {
            return Err(AppError {
                message: format!("Class subject code already exists: {}", existing.code),
            });
        }

        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);

        sqlx::query(
            r#"
            INSERT INTO class_subjects (
              id, school_id, class_id, teacher_id, teacher_user_id, main_subject_id,
              name, code, description, category, estimated_hours, credits, created_by, disable,
              created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, NULL, $5, $6, $7, $8, $9, $10, $11, $12, $13, COALESCE($14, now()), COALESCE($15, now()))
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(&class_id)
        .bind(dto.teacher_id.as_ref().map(|id| id.to_hex()))
        .bind(dto.main_subject_id.as_ref().map(|id| id.to_hex()))
        .bind(&dto.name)
        .bind(&dto.code)
        .bind(&dto.description)
        .bind(Self::enum_to_string(&dto.category))
        .bind(dto.estimated_hours)
        .bind(dto.credits)
        .bind(dto.created_by.as_ref().map(|id| id.to_hex()))
        .bind(dto.disable)
        .bind(dto.created_at)
        .bind(dto.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if let Some(topics) = &dto.topics {
            self.sync_topics(&id, topics).await?;
        }

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<ClassSubjectQuery>,
    ) -> Result<ClassSubject, AppError> {
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
            Some(row) => self.subject_from_row(row).await,
            None => Err(AppError {
                message: "Class subject not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ClassSubjectQuery>,
    ) -> Result<Paginated<ClassSubject>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM class_subjects WHERE deleted_at IS NULL",
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
            data.push(self.subject_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update_subject(
        &self,
        id: &IdType,
        update: &ClassSubjectPartial,
    ) -> Result<ClassSubject, AppError> {
        let existing = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        let school_id = existing.school_id.as_ref().map(|id| id.to_hex());

        if let Some(code) = update.code.clone() {
            if existing.code != code
                && self
                    .code_exists(&code, school_id.as_deref(), Some(&id_string))
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("Code '{}' already exists", code),
                });
            }
        }

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE class_subjects SET updated_at = now()");
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
        set_optional!(credits, "credits");
        set_optional!(disable, "disable");

        if let Some(value) = update.teacher_id {
            sql.push(", teacher_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.class_id {
            sql.push(", class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.main_subject_id {
            sql.push(", main_subject_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.created_by {
            sql.push(", created_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update.category.clone() {
            sql.push(", category = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update.estimated_hours {
            sql.push(", estimated_hours = ").push_bind(value);
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
        if let Some(Some(topics)) = &update.topics {
            self.sync_topics(&id_string, topics).await?;
        }
        if !touched && update.topics.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.find_one(Some(id), None).await
    }

    pub async fn delete_subject(&self, id: &IdType) -> Result<ClassSubject, AppError> {
        let existing = self.find_one(Some(id), None).await?;
        let id_string = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE class_subjects SET deleted_at = now(), updated_at = now() WHERE id = $1",
        )
        .bind(id_string)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(existing)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ClassSubjectQuery>,
    ) -> Result<Paginated<ClassSubjectWithRelations>, AppError> {
        let subjects = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(subjects.data.len());
        for subject in subjects.data {
            data.push(self.with_relations(subject).await?);
        }

        Ok(Paginated {
            data,
            total: subjects.total,
            total_pages: subjects.total_pages,
            current_page: subjects.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<ClassSubjectQuery>,
    ) -> Result<ClassSubjectWithRelations, AppError> {
        let subject = self.find_one(id, query).await?;
        self.with_relations(subject).await
    }

    async fn with_relations(
        &self,
        subject: ClassSubject,
    ) -> Result<ClassSubjectWithRelations, AppError> {
        let school_service = SchoolService::new(&self.pool);
        let class_service = ClassService::new(&self.pool);
        let teacher_service = TeacherService::new(&self.pool);

        let school = match subject.school_id {
            Some(school_id) => school_service
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };
        let class = match subject.class_id {
            Some(class_id) => class_service
                .find_one(Some(&IdType::from_object_id(class_id)), None)
                .await
                .ok(),
            None => None,
        };
        let teacher = match subject.teacher_id {
            Some(teacher_id) => teacher_service
                .find_one(Some(&IdType::from_object_id(teacher_id)), None)
                .await
                .ok(),
            None => None,
        };

        Ok(ClassSubjectWithRelations {
            subject,
            main_template_subject: None,
            school,
            class,
            teacher,
        })
    }

    pub async fn count_subject(
        &self,
        filter: Option<String>,
        query: Option<ClassSubjectQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub async fn create_many(
        &self,
        dtos: Vec<ClassSubject>,
    ) -> Result<Vec<ClassSubject>, AppError> {
        let mut subjects = Vec::with_capacity(dtos.len());
        for dto in dtos {
            subjects.push(self.create(dto).await?);
        }
        Ok(subjects)
    }
}
