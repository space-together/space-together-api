use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        assignment::{
            Assignment, AssignmentPartial, AssignmentStatus, AssignmentWithRelations, Submission,
            SubmissionPartial, SubmissionStatus, SubmissionWithRelations,
        },
        common_details::Paginated,
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::cloudinary_service::CloudinaryService,
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct AssignmentQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub class_id: Option<String>,
    pub assignment_id: Option<String>,
    pub student_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
pub struct CountResult {
    pub count: u64,
}

#[derive(Debug, Clone)]
pub struct StudentSubmissionContext {
    pub student_id: ObjectId,
    pub class_id: Option<ObjectId>,
}

impl AssignmentQuery {
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
            assignment_id: None,
            student_id: None,
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

pub struct AssignmentService {
    pub pool: PgPool,
}

impl AssignmentService {
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

    fn enum_to_string<T: Serialize>(value: &T) -> String {
        match serde_json::to_value(value).ok() {
            Some(serde_json::Value::String(value)) => value,
            Some(other) => other.to_string(),
            None => String::new(),
        }
    }

    fn enum_from_string<T: DeserializeOwned>(value: Option<String>, fallback: T) -> T {
        value
            .and_then(|raw| serde_json::from_value(serde_json::Value::String(raw)).ok())
            .unwrap_or(fallback)
    }

    fn assignment_select_sql() -> &'static str {
        r#"
        SELECT
          a.id, a.school_id, a.class_id,
          COALESCE(a.subject_id, a.class_subject_id) AS subject_id,
          a.teacher_id, a.title, a.description, a.instructions,
          COALESCE(a.due_date, a.due_at) AS due_date,
          COALESCE(a.max_score, 100)::DOUBLE PRECISION AS max_score,
          COALESCE(a.allow_late_submission, false) AS allow_late_submission,
          a.attachment_url, a.attachment_id, a.status,
          COALESCE(a.auto_grade_enabled, false) AS auto_grade_enabled,
          a.deleted_at, a.created_at, a.updated_at,
          (
            SELECT count(*) FROM submissions s
            WHERE s.assignment_id = a.id AND s.deleted_at IS NULL
          )::BIGINT AS submission_count,
          (
            SELECT count(*) FROM student_school_enrollments e
            WHERE e.class_id = a.class_id AND e.deleted_at IS NULL AND e.is_active = true
          )::BIGINT AS total_students
        FROM assignments a
        WHERE a.deleted_at IS NULL
        "#
    }

    fn submission_select_sql() -> &'static str {
        r#"
        SELECT
          s.id, s.assignment_id, s.student_id, s.file_url, s.file_id, s.comment,
          COALESCE(s.is_late, false) AS is_late,
          s.score::DOUBLE PRECISION AS score,
          s.feedback, s.feedback_file_url, s.feedback_file_id, s.graded_at,
          s.graded_by, s.status,
          s.auto_grade_score::DOUBLE PRECISION AS auto_grade_score,
          s.ai_feedback, s.deleted_at,
          COALESCE(s.submitted_at, s.created_at) AS submitted_at,
          s.updated_at
        FROM submissions s
        WHERE s.deleted_at IS NULL
        "#
    }

    fn push_assignment_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a AssignmentQuery>,
    ) -> Result<(), AppError> {
        let Some(query) = query else {
            return Ok(());
        };

        if !query.by_ids.is_empty() {
            sql.push(" AND a.id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                parse_object_id_value(id)?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }
        if let Some(school_id) = &query.school_id {
            parse_object_id_value(school_id)?;
            sql.push(" AND a.school_id = ").push_bind(school_id);
        }
        if let Some(class_id) = &query.class_id {
            parse_object_id_value(class_id)?;
            sql.push(" AND a.class_id = ").push_bind(class_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND a.id = ").push_bind(value);
                }
                "school_id" | "class_id" | "teacher_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND a.").push(field).push(" = ").push_bind(value);
                }
                "subject_id" | "class_subject_id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND COALESCE(a.subject_id, a.class_subject_id) = ")
                        .push_bind(value);
                }
                "title" | "status" => {
                    sql.push(" AND lower(a.")
                        .push(field)
                        .push(") = lower(")
                        .push_bind(value)
                        .push(")");
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported assignment filter field: {}", field),
                    });
                }
            }
        }
        Ok(())
    }

    fn push_submission_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a AssignmentQuery>,
    ) -> Result<(), AppError> {
        let Some(query) = query else {
            return Ok(());
        };

        if !query.by_ids.is_empty() {
            sql.push(" AND s.id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                parse_object_id_value(id)?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }
        if let Some(assignment_id) = &query.assignment_id {
            parse_object_id_value(assignment_id)?;
            sql.push(" AND s.assignment_id = ").push_bind(assignment_id);
        }
        if let Some(student_id) = &query.student_id {
            parse_object_id_value(student_id)?;
            sql.push(" AND s.student_id = ").push_bind(student_id);
        }
        if let Some(school_id) = &query.school_id {
            parse_object_id_value(school_id)?;
            sql.push(" AND s.school_id = ").push_bind(school_id);
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND s.id = ").push_bind(value);
                }
                "assignment_id" | "student_id" | "graded_by" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND s.").push(field).push(" = ").push_bind(value);
                }
                "status" => {
                    sql.push(" AND lower(s.status) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported submission filter field: {}", field),
                    });
                }
            }
        }
        Ok(())
    }

    fn push_assignment_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(a.title, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(a.description, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(a.status, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(a.id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    fn push_submission_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(coalesce(s.status, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(s.feedback, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(s.id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    fn assignment_from_row(row: &PgRow) -> Result<Assignment, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let deleted_at: Option<DateTime<Utc>> = row.try_get("deleted_at").ok().flatten();

        Ok(Assignment {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            subject_id: Self::parse_oid_opt(
                row.try_get("subject_id").ok().flatten(),
                "subject_id",
            )?,
            teacher_id: Self::parse_oid_opt(
                row.try_get("teacher_id").ok().flatten(),
                "teacher_id",
            )?,
            title: row.try_get("title").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            instructions: row.try_get("instructions").ok().flatten(),
            due_date: row.try_get("due_date").map_err(Self::db_error)?,
            max_score: row.try_get("max_score").map_err(Self::db_error)?,
            allow_late_submission: row.try_get("allow_late_submission").ok().unwrap_or(false),
            attachment_url: row.try_get("attachment_url").ok().flatten(),
            attachment_id: row.try_get("attachment_id").ok().flatten(),
            status: Self::enum_from_string(
                row.try_get("status").ok().flatten(),
                AssignmentStatus::Published,
            ),
            auto_grade_enabled: row.try_get("auto_grade_enabled").ok().unwrap_or(false),
            is_deleted: deleted_at.is_some(),
            deleted_at,
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    fn assignment_with_relations_from_row(row: PgRow) -> Result<AssignmentWithRelations, AppError> {
        Ok(AssignmentWithRelations {
            assignment: Self::assignment_from_row(&row)?,
            teacher: None,
            subject: None,
            class: None,
            submission_count: row.try_get("submission_count").ok(),
            total_students: row.try_get("total_students").ok(),
        })
    }

    fn submission_from_row(row: &PgRow) -> Result<Submission, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let deleted_at: Option<DateTime<Utc>> = row.try_get("deleted_at").ok().flatten();

        Ok(Submission {
            id: Some(Self::parse_oid(&id, "id")?),
            assignment_id: Self::parse_oid_opt(
                row.try_get("assignment_id").ok().flatten(),
                "assignment_id",
            )?,
            student_id: Self::parse_oid_opt(
                row.try_get("student_id").ok().flatten(),
                "student_id",
            )?,
            file_url: row.try_get("file_url").ok().flatten(),
            file_id: row.try_get("file_id").ok().flatten(),
            comment: row.try_get("comment").ok().flatten(),
            is_late: row.try_get("is_late").ok().unwrap_or(false),
            score: row.try_get("score").ok().flatten(),
            feedback: row.try_get("feedback").ok().flatten(),
            feedback_file_url: row.try_get("feedback_file_url").ok().flatten(),
            feedback_file_id: row.try_get("feedback_file_id").ok().flatten(),
            graded_at: row.try_get("graded_at").ok().flatten(),
            graded_by: Self::parse_oid_opt(row.try_get("graded_by").ok().flatten(), "graded_by")?,
            status: Self::enum_from_string(
                row.try_get("status").ok().flatten(),
                SubmissionStatus::Submitted,
            ),
            auto_grade_score: row.try_get("auto_grade_score").ok().flatten(),
            ai_feedback: row.try_get("ai_feedback").ok().flatten(),
            is_deleted: deleted_at.is_some(),
            deleted_at,
            submitted_at: row.try_get("submitted_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    fn submission_with_relations_from_row(row: PgRow) -> Result<SubmissionWithRelations, AppError> {
        Ok(SubmissionWithRelations {
            submission: Self::submission_from_row(&row)?,
            student: None,
            assignment: None,
            graded_by_teacher: None,
        })
    }

    pub async fn is_feature_enabled(
        &self,
        school_id: &str,
        feature_name: &str,
    ) -> Result<bool, AppError> {
        let enabled: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT enabled
            FROM school_feature_flags
            WHERE school_id = $1 AND feature_name = $2
            "#,
        )
        .bind(school_id)
        .bind(feature_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(enabled.unwrap_or(true))
    }

    pub async fn find_teacher_id_for_user(
        &self,
        user_id: &str,
        school_id: Option<&str>,
    ) -> Result<Option<ObjectId>, AppError> {
        parse_object_id_value(user_id)?;
        let mut query = QueryBuilder::<Postgres>::new("SELECT id FROM teachers WHERE user_id = ");
        query.push_bind(user_id).push(" AND deleted_at IS NULL");
        if let Some(school_id) = school_id {
            parse_object_id_value(school_id)?;
            query.push(" AND school_id = ").push_bind(school_id);
        }
        query.push(" ORDER BY updated_at DESC LIMIT 1");

        let id: Option<String> = query
            .build_query_scalar()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        id.map(|id| Self::parse_oid(&id, "teacher_id")).transpose()
    }

    pub async fn teacher_belongs_to_user(
        &self,
        teacher_id: &ObjectId,
        user_id: &str,
    ) -> Result<bool, AppError> {
        parse_object_id_value(user_id)?;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM teachers WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
        )
        .bind(teacher_id.to_hex())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(exists)
    }

    pub async fn find_student_for_user(
        &self,
        user_id: &str,
        school_id: Option<&str>,
    ) -> Result<Option<StudentSubmissionContext>, AppError> {
        parse_object_id_value(user_id)?;
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT sp.id AS student_id, sse.class_id
            FROM student_profiles sp
            JOIN student_school_enrollments sse ON sse.student_id = sp.id
            WHERE sp.user_id =
            "#,
        );
        query
            .push_bind(user_id)
            .push(" AND sp.deleted_at IS NULL AND sse.deleted_at IS NULL AND sse.is_active = true");
        if let Some(school_id) = school_id {
            parse_object_id_value(school_id)?;
            query.push(" AND sse.school_id = ").push_bind(school_id);
        }
        query.push(" ORDER BY sse.updated_at DESC LIMIT 1");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        row.map(|row| {
            let student_id: String = row.try_get("student_id").map_err(Self::db_error)?;
            Ok(StudentSubmissionContext {
                student_id: Self::parse_oid(&student_id, "student_id")?,
                class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            })
        })
        .transpose()
    }

    pub async fn student_belongs_to_user(
        &self,
        student_id: &ObjectId,
        user_id: &str,
    ) -> Result<bool, AppError> {
        parse_object_id_value(user_id)?;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM student_profiles WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
        )
        .bind(student_id.to_hex())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(exists)
    }

    pub async fn create_assignment(&self, dto: Assignment) -> Result<Assignment, AppError> {
        self.ensure_indexes().await?;

        if dto.title.trim().is_empty() {
            return Err(AppError {
                message: "Assignment title is required".into(),
            });
        }
        if dto.max_score <= 0.0 {
            return Err(AppError {
                message: "Max score must be greater than 0".into(),
            });
        }

        if let (Some(teacher_id), Some(subject_id)) = (dto.teacher_id, dto.subject_id) {
            let assigned: Option<bool> = sqlx::query_scalar(
                r#"
                SELECT
                  CASE
                    WHEN cs.teacher_user_id IS NULL THEN true
                    WHEN t.user_id = cs.teacher_user_id THEN true
                    WHEN EXISTS (
                      SELECT 1 FROM teacher_subjects ts
                      WHERE ts.teacher_id = t.id AND ts.class_subject_id = cs.id
                    ) THEN true
                    ELSE false
                  END
                FROM class_subjects cs
                LEFT JOIN teachers t ON t.id = $1
                WHERE cs.id = $2 AND cs.deleted_at IS NULL
                "#,
            )
            .bind(teacher_id.to_hex())
            .bind(subject_id.to_hex())
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

            if assigned == Some(false) {
                return Err(AppError {
                    message: "Teacher is not assigned to this subject".into(),
                });
            }
        }

        let mut assignment = dto;
        if let Some(attachment_data) = assignment.attachment_url.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&attachment_data)
                .await
                .map_err(|e| AppError { message: e })?;
            assignment.attachment_id = Some(cloud_res.public_id);
            assignment.attachment_url = Some(cloud_res.secure_url);
        }

        let school_id = assignment
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let id = assignment
            .id
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let teacher_user_id: Option<String> = match assignment.teacher_id {
            Some(teacher_id) => sqlx::query_scalar("SELECT user_id FROM teachers WHERE id = $1")
                .bind(teacher_id.to_hex())
                .fetch_optional(&self.pool)
                .await
                .map_err(Self::db_error)?,
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO assignments (
              id, school_id, class_id, class_subject_id, subject_id, teacher_id, teacher_user_id,
              title, description, instructions, due_at, due_date, max_score,
              allow_late_submission, attachment_url, attachment_id, status, auto_grade_enabled,
              deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $4, $5, $6, $7, $8, $9, $10, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(assignment.class_id.as_ref().map(|id| id.to_hex()))
        .bind(assignment.subject_id.as_ref().map(|id| id.to_hex()))
        .bind(assignment.teacher_id.as_ref().map(|id| id.to_hex()))
        .bind(teacher_user_id)
        .bind(&assignment.title)
        .bind(&assignment.description)
        .bind(&assignment.instructions)
        .bind(assignment.due_date)
        .bind(assignment.max_score)
        .bind(assignment.allow_late_submission)
        .bind(&assignment.attachment_url)
        .bind(&assignment.attachment_id)
        .bind(Self::enum_to_string(&assignment.status))
        .bind(assignment.auto_grade_enabled)
        .bind(if assignment.is_deleted { Some(Utc::now()) } else { assignment.deleted_at })
        .bind(assignment.created_at)
        .bind(assignment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one_assignment(Some(&IdType::from_string(id)), None)
            .await
    }

    pub async fn find_one_assignment(
        &self,
        id: Option<&IdType>,
        query: Option<AssignmentQuery>,
    ) -> Result<Assignment, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::assignment_select_sql());
        Self::push_assignment_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            sql.push(" AND a.id = ").push_bind(Self::id_to_string(id)?);
        }
        sql.push(" ORDER BY a.updated_at DESC LIMIT 1");
        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        row.as_ref()
            .map(Self::assignment_from_row)
            .transpose()?
            .ok_or(AppError {
                message: "Assignment not found".into(),
            })
    }

    pub async fn find_one_assignment_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<AssignmentQuery>,
    ) -> Result<AssignmentWithRelations, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::assignment_select_sql());
        Self::push_assignment_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            sql.push(" AND a.id = ").push_bind(Self::id_to_string(id)?);
        }
        sql.push(" ORDER BY a.updated_at DESC LIMIT 1");
        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        row.map(Self::assignment_with_relations_from_row)
            .transpose()?
            .ok_or(AppError {
                message: "Assignment not found".into(),
            })
    }

    pub async fn get_all_assignments(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<AssignmentQuery>,
    ) -> Result<Paginated<Assignment>, AppError> {
        let page = self
            .get_all_assignments_with_relations(filter, limit, skip, query)
            .await?;
        Ok(Paginated {
            data: page.data.into_iter().map(|item| item.assignment).collect(),
            total: page.total,
            total_pages: page.total_pages,
            current_page: page.current_page,
        })
    }

    pub async fn get_all_assignments_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<AssignmentQuery>,
    ) -> Result<Paginated<AssignmentWithRelations>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM assignments a WHERE a.deleted_at IS NULL",
        );
        Self::push_assignment_filters(&mut count_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_assignment_search(&mut count_query, search_like);
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data_query = QueryBuilder::<Postgres>::new(Self::assignment_select_sql());
        Self::push_assignment_filters(&mut data_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_assignment_search(&mut data_query, search_like);
        }
        data_query
            .push(" ORDER BY a.updated_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);
        let rows = data_query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let data = rows
            .into_iter()
            .map(Self::assignment_with_relations_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update_assignment(
        &self,
        id: &IdType,
        update: &AssignmentPartial,
    ) -> Result<Assignment, AppError> {
        let existing = self.find_one_assignment(Some(id), None).await?;
        if let Some(ref title) = update.title {
            if title.trim().is_empty() {
                return Err(AppError {
                    message: "Assignment title cannot be empty".into(),
                });
            }
        }
        if let Some(max_score) = update.max_score {
            if max_score <= 0.0 {
                return Err(AppError {
                    message: "Max score must be greater than 0".into(),
                });
            }
        }

        let mut update_data = update.clone();
        if let Some(new_attachment) = update.attachment_url.clone().flatten() {
            if Some(new_attachment.clone()) != existing.attachment_url {
                if let Some(old_attachment_id) = existing.attachment_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_attachment_id)
                        .await
                        .ok();
                }
                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_attachment)
                    .await
                    .map_err(|e| AppError { message: e })?;
                update_data.attachment_id = Some(Some(cloud_res.public_id));
                update_data.attachment_url = Some(Some(cloud_res.secure_url));
            }
        }

        let id_string = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE assignments SET updated_at = now()");
        let mut touched = false;

        macro_rules! set_value {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }
        macro_rules! set_nullable {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }

        set_value!(title, "title");
        set_nullable!(description, "description");
        set_nullable!(instructions, "instructions");
        set_value!(due_date, "due_date");
        set_value!(max_score, "max_score");
        set_value!(allow_late_submission, "allow_late_submission");
        set_nullable!(attachment_url, "attachment_url");
        set_nullable!(attachment_id, "attachment_id");
        set_value!(auto_grade_enabled, "auto_grade_enabled");
        if let Some(value) = update_data.due_date {
            sql.push(", due_at = ").push_bind(value);
            touched = true;
        }
        if let Some(value) = update_data.status {
            sql.push(", status = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update_data.school_id {
            sql.push(", school_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.class_id {
            sql.push(", class_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.subject_id {
            let subject_id = value.map(|id| id.to_hex());
            sql.push(", subject_id = ")
                .push_bind(subject_id.clone())
                .push(", class_subject_id = ")
                .push_bind(subject_id);
            touched = true;
        }
        if let Some(value) = update_data.teacher_id {
            let teacher_id = value.map(|id| id.to_hex());
            let teacher_user_id: Option<String> = match teacher_id.as_deref() {
                Some(id) => sqlx::query_scalar("SELECT user_id FROM teachers WHERE id = $1")
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(Self::db_error)?,
                None => None,
            };
            sql.push(", teacher_id = ")
                .push_bind(teacher_id)
                .push(", teacher_user_id = ")
                .push_bind(teacher_user_id);
            touched = true;
        }
        if let Some(value) = update_data.is_deleted {
            sql.push(", deleted_at = ");
            if value {
                sql.push("now()");
            } else {
                sql.push("NULL");
            }
            touched = true;
        }
        if let Some(value) = update_data.deleted_at {
            sql.push(", deleted_at = ").push_bind(value);
            touched = true;
        }

        if !touched {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }
        sql.push(" WHERE id = ")
            .push_bind(id_string)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one_assignment(Some(id), None).await
    }

    pub async fn delete_assignment(&self, id: &IdType) -> Result<Assignment, AppError> {
        let assignment = self.find_one_assignment(Some(id), None).await?;
        sqlx::query("UPDATE assignments SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(assignment)
    }

    pub async fn count_assignments(
        &self,
        filter: Option<String>,
        query: Option<AssignmentQuery>,
    ) -> Result<CountResult, AppError> {
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));
        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM assignments a WHERE a.deleted_at IS NULL",
        );
        Self::push_assignment_filters(&mut count_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_assignment_search(&mut count_query, search_like);
        }
        let count: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(CountResult {
            count: count.max(0) as u64,
        })
    }

    pub async fn validate_submission_deadline(
        &self,
        assignment: &Assignment,
    ) -> Result<bool, AppError> {
        let is_late = Utc::now() > assignment.due_date;
        if is_late && !assignment.allow_late_submission {
            return Err(AppError {
                message: "Submission deadline has passed and late submissions are not allowed"
                    .into(),
            });
        }
        Ok(is_late)
    }

    pub async fn create_submission(&self, dto: Submission) -> Result<Submission, AppError> {
        self.ensure_indexes().await?;
        let assignment_id = dto
            .assignment_id
            .map(|id| IdType::ObjectId(id))
            .ok_or(AppError {
                message: "Assignment ID is required".into(),
            })?;
        let assignment = self.find_one_assignment(Some(&assignment_id), None).await?;
        if !matches!(assignment.status, AssignmentStatus::Published) {
            return Err(AppError {
                message: "Assignment is not published".into(),
            });
        }
        let is_late = self.validate_submission_deadline(&assignment).await?;
        let assignment_id_string = Self::id_to_string(&assignment_id)?;
        let student_id = dto
            .student_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "Student ID is required".into(),
            })?;

        let existing: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM submissions WHERE assignment_id = $1 AND student_id = $2 AND deleted_at IS NULL)",
        )
        .bind(&assignment_id_string)
        .bind(&student_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;
        if existing {
            return Err(AppError {
                message: "You have already submitted this assignment. Use update instead.".into(),
            });
        }

        let mut submission = dto;
        submission.is_late = is_late;
        if let Some(file_data) = submission.file_url.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&file_data)
                .await
                .map_err(|e| AppError { message: e })?;
            submission.file_id = Some(cloud_res.public_id);
            submission.file_url = Some(cloud_res.secure_url);
        }

        let id = submission
            .id
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let school_id = assignment
            .school_id
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "Assignment has no school_id".into(),
            })?;

        sqlx::query(
            r#"
            INSERT INTO submissions (
              id, school_id, assignment_id, student_id, file_url, file_id, comment,
              is_late, score, feedback, feedback_file_url, feedback_file_id,
              graded_at, graded_by, status, auto_grade_score, ai_feedback, deleted_at,
              submitted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $19, $20)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(&assignment_id_string)
        .bind(&student_id)
        .bind(&submission.file_url)
        .bind(&submission.file_id)
        .bind(&submission.comment)
        .bind(submission.is_late)
        .bind(submission.score)
        .bind(&submission.feedback)
        .bind(&submission.feedback_file_url)
        .bind(&submission.feedback_file_id)
        .bind(submission.graded_at)
        .bind(submission.graded_by.as_ref().map(|id| id.to_hex()))
        .bind(Self::enum_to_string(&submission.status))
        .bind(submission.auto_grade_score)
        .bind(&submission.ai_feedback)
        .bind(if submission.is_deleted { Some(Utc::now()) } else { submission.deleted_at })
        .bind(submission.submitted_at)
        .bind(submission.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one_submission(Some(&IdType::from_string(id)), None)
            .await
    }

    pub async fn find_one_submission(
        &self,
        id: Option<&IdType>,
        query: Option<AssignmentQuery>,
    ) -> Result<Submission, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::submission_select_sql());
        Self::push_submission_filters(&mut sql, query.as_ref())?;
        if let Some(id) = id {
            sql.push(" AND s.id = ").push_bind(Self::id_to_string(id)?);
        }
        sql.push(" ORDER BY s.updated_at DESC LIMIT 1");
        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        row.as_ref()
            .map(Self::submission_from_row)
            .transpose()?
            .ok_or(AppError {
                message: "Submission not found".into(),
            })
    }

    pub async fn get_all_submissions(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<AssignmentQuery>,
    ) -> Result<Paginated<Submission>, AppError> {
        let page = self
            .get_all_submissions_with_relations(filter, limit, skip, query)
            .await?;
        Ok(Paginated {
            data: page.data.into_iter().map(|item| item.submission).collect(),
            total: page.total,
            total_pages: page.total_pages,
            current_page: page.current_page,
        })
    }

    pub async fn get_all_submissions_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<AssignmentQuery>,
    ) -> Result<Paginated<SubmissionWithRelations>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM submissions s WHERE s.deleted_at IS NULL",
        );
        Self::push_submission_filters(&mut count_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_submission_search(&mut count_query, search_like);
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data_query = QueryBuilder::<Postgres>::new(Self::submission_select_sql());
        Self::push_submission_filters(&mut data_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_submission_search(&mut data_query, search_like);
        }
        data_query
            .push(" ORDER BY s.updated_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);
        let rows = data_query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let data = rows
            .into_iter()
            .map(Self::submission_with_relations_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update_submission(
        &self,
        id: &IdType,
        update: &SubmissionPartial,
    ) -> Result<Submission, AppError> {
        let existing = self.find_one_submission(Some(id), None).await?;
        if let Some(score) = update.score.flatten() {
            if let Some(assignment_id) = existing.assignment_id {
                let assignment = self
                    .find_one_assignment(Some(&IdType::ObjectId(assignment_id)), None)
                    .await?;
                if score > assignment.max_score {
                    return Err(AppError {
                        message: format!(
                            "Score cannot exceed max score of {}",
                            assignment.max_score
                        ),
                    });
                }
            }
        }

        let mut update_data = update.clone();
        if let Some(new_file) = update.file_url.clone().flatten() {
            if Some(new_file.clone()) != existing.file_url {
                if let Some(old_file_id) = existing.file_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_file_id)
                        .await
                        .ok();
                }
                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_file)
                    .await
                    .map_err(|e| AppError { message: e })?;
                update_data.file_id = Some(Some(cloud_res.public_id));
                update_data.file_url = Some(Some(cloud_res.secure_url));
            }
        }
        if let Some(feedback_file) = update.feedback_file_url.clone().flatten() {
            if Some(feedback_file.clone()) != existing.feedback_file_url {
                if let Some(old_feedback_file_id) = existing.feedback_file_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_feedback_file_id)
                        .await
                        .ok();
                }
                let cloud_res = CloudinaryService::upload_to_cloudinary(&feedback_file)
                    .await
                    .map_err(|e| AppError { message: e })?;
                update_data.feedback_file_id = Some(Some(cloud_res.public_id));
                update_data.feedback_file_url = Some(Some(cloud_res.secure_url));
            }
        }

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE submissions SET updated_at = now()");
        let mut touched = false;

        macro_rules! set_nullable {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }
        macro_rules! set_value {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched = true;
                }
            };
        }

        set_nullable!(file_url, "file_url");
        set_nullable!(file_id, "file_id");
        set_nullable!(comment, "comment");
        set_value!(is_late, "is_late");
        set_nullable!(score, "score");
        set_nullable!(feedback, "feedback");
        set_nullable!(feedback_file_url, "feedback_file_url");
        set_nullable!(feedback_file_id, "feedback_file_id");
        set_nullable!(graded_at, "graded_at");
        set_nullable!(auto_grade_score, "auto_grade_score");
        set_nullable!(ai_feedback, "ai_feedback");
        if let Some(value) = update_data.assignment_id {
            sql.push(", assignment_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.student_id {
            sql.push(", student_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.graded_by {
            sql.push(", graded_by = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched = true;
        }
        if let Some(value) = update_data.status {
            sql.push(", status = ")
                .push_bind(Self::enum_to_string(&value));
            touched = true;
        }
        if let Some(value) = update_data.is_deleted {
            sql.push(", deleted_at = ");
            if value {
                sql.push("now()");
            } else {
                sql.push("NULL");
            }
            touched = true;
        }
        if let Some(value) = update_data.deleted_at {
            sql.push(", deleted_at = ").push_bind(value);
            touched = true;
        }
        if !touched {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }
        sql.push(" WHERE id = ")
            .push_bind(Self::id_to_string(id)?)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one_submission(Some(id), None).await
    }

    pub async fn grade_submission(
        &self,
        submission_id: &IdType,
        score: f64,
        feedback: Option<String>,
        feedback_file: Option<String>,
        graded_by: ObjectId,
    ) -> Result<Submission, AppError> {
        let submission = self.find_one_submission(Some(submission_id), None).await?;
        if let Some(assignment_id) = submission.assignment_id {
            let assignment = self
                .find_one_assignment(Some(&IdType::ObjectId(assignment_id)), None)
                .await?;
            if score > assignment.max_score {
                return Err(AppError {
                    message: format!("Score cannot exceed max score of {}", assignment.max_score),
                });
            }
        }

        let update = SubmissionPartial {
            score: Some(Some(score)),
            feedback: Some(feedback),
            feedback_file_url: Some(feedback_file),
            graded_by: Some(Some(graded_by)),
            graded_at: Some(Some(Utc::now())),
            status: Some(SubmissionStatus::Graded),
            ..Default::default()
        };

        self.update_submission(submission_id, &update).await
    }

    pub async fn delete_submission(&self, id: &IdType) -> Result<Submission, AppError> {
        let submission = self.find_one_submission(Some(id), None).await?;
        sqlx::query("UPDATE submissions SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(submission)
    }

    #[allow(dead_code)]
    pub async fn trigger_auto_grading(&self, _submission_id: &IdType) -> Result<(), AppError> {
        Ok(())
    }
}
