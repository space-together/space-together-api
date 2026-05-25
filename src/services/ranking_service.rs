use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::{
    domain::student_term_result::StudentTermResult, errors::AppError, utils::object_id::ObjectId,
};

pub struct RankingService {
    pub pool: PgPool,
}

impl RankingService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(error: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", error),
        }
    }

    fn parse_oid(raw: &str, field: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(raw).map_err(|e| AppError {
            message: format!("Invalid {} ObjectId-compatible ID: {}", field, e),
        })
    }

    fn parse_oid_opt(raw: Option<String>, field: &str) -> Result<Option<ObjectId>, AppError> {
        raw.map(|value| Self::parse_oid(&value, field)).transpose()
    }

    fn result_from_row(row: PgRow) -> Result<StudentTermResult, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(StudentTermResult {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            student_id: Self::parse_oid_opt(
                row.try_get("student_id").ok().flatten(),
                "student_id",
            )?,
            class_id: Self::parse_oid_opt(row.try_get("class_id").ok().flatten(), "class_id")?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            term_id: row.try_get("term_id").ok().flatten(),
            exam_id: Self::parse_oid_opt(row.try_get("exam_id").ok().flatten(), "exam_id")?,
            subject_results: Vec::new(),
            total_score: row.try_get("total_score").ok().unwrap_or_default(),
            total_max_score: row.try_get("total_max_score").ok().unwrap_or_default(),
            average_percentage: row.try_get("average_percentage").ok().unwrap_or_default(),
            gpa: row.try_get("gpa").ok().unwrap_or_default(),
            total_credits: row.try_get("total_credits").ok().flatten(),
            grade: row
                .try_get::<Option<String>, _>("grade")
                .ok()
                .flatten()
                .unwrap_or_default(),
            rank_in_class: row.try_get("rank_in_class").ok().flatten(),
            total_students: row.try_get("total_students").ok().flatten(),
            calculated_at: row
                .try_get::<Option<DateTime<Utc>>, _>("calculated_at")
                .ok()
                .flatten(),
            is_finalized: row.try_get("is_finalized").ok().unwrap_or(false),
        })
    }

    fn select_sql(order_by: &str) -> String {
        format!(
            r#"
            SELECT id, school_id, student_id, class_id, education_year_id,
                   COALESCE(term_id, term) AS term_id,
                   exam_id,
                   total_score::DOUBLE PRECISION AS total_score,
                   COALESCE(total_max_score, 0)::DOUBLE PRECISION AS total_max_score,
                   COALESCE(average_percentage, average_score, 0)::DOUBLE PRECISION AS average_percentage,
                   COALESCE(gpa, average_score, 0)::DOUBLE PRECISION AS gpa,
                   total_credits,
                   COALESCE(grade, status, '') AS grade,
                   COALESCE(rank_in_class, rank) AS rank_in_class,
                   total_students,
                   COALESCE(calculated_at, updated_at, created_at) AS calculated_at,
                   COALESCE(is_finalized, false) AS is_finalized
            FROM student_term_results
            WHERE class_id = $1 AND exam_id = $2 AND deleted_at IS NULL
            {order_by}
            "#
        )
    }

    pub async fn calculate_rankings(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let mut results = self
            .fetch_rankings(
                class_id,
                exam_id,
                "ORDER BY gpa DESC, average_percentage DESC",
                None,
            )
            .await?;
        let total_students = results.len() as i32;
        let mut current_rank = 1;
        let mut previous_gpa: Option<f64> = None;

        for (index, result) in results.iter_mut().enumerate() {
            if let Some(prev_gpa) = previous_gpa {
                if (result.gpa - prev_gpa).abs() >= 0.001 {
                    current_rank = (index as i32) + 1;
                }
            }
            result.rank_in_class = Some(current_rank);
            result.total_students = Some(total_students);
            previous_gpa = Some(result.gpa);

            if let Some(result_id) = result.id {
                sqlx::query(
                    r#"
                    UPDATE student_term_results
                    SET rank_in_class = $1, rank = $1, total_students = $2, updated_at = now()
                    WHERE id = $3
                    "#,
                )
                .bind(current_rank)
                .bind(total_students)
                .bind(result_id.to_hex())
                .execute(&self.pool)
                .await
                .ok();
            }
        }

        Ok(results)
    }

    pub async fn get_class_rankings(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        self.fetch_rankings(
            class_id,
            exam_id,
            "ORDER BY rank_in_class ASC NULLS LAST",
            None,
        )
        .await
    }

    pub async fn get_top_students(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        limit: i64,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        self.fetch_rankings(
            class_id,
            exam_id,
            "ORDER BY gpa DESC, average_percentage DESC",
            Some(limit),
        )
        .await
    }

    pub async fn get_at_risk_students(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        gpa_threshold: f64,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, school_id, student_id, class_id, education_year_id,
                   COALESCE(term_id, term) AS term_id,
                   exam_id,
                   total_score::DOUBLE PRECISION AS total_score,
                   COALESCE(total_max_score, 0)::DOUBLE PRECISION AS total_max_score,
                   COALESCE(average_percentage, average_score, 0)::DOUBLE PRECISION AS average_percentage,
                   COALESCE(gpa, average_score, 0)::DOUBLE PRECISION AS gpa,
                   total_credits,
                   COALESCE(grade, status, '') AS grade,
                   COALESCE(rank_in_class, rank) AS rank_in_class,
                   total_students,
                   COALESCE(calculated_at, updated_at, created_at) AS calculated_at,
                   COALESCE(is_finalized, false) AS is_finalized
            FROM student_term_results
            WHERE class_id = $1 AND exam_id = $2
              AND COALESCE(gpa, average_score, 0) < $3
              AND deleted_at IS NULL
            ORDER BY COALESCE(gpa, average_score, 0) ASC
            "#,
        )
        .bind(class_id.to_hex())
        .bind(exam_id.to_hex())
        .bind(gpa_threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter().map(Self::result_from_row).collect()
    }

    async fn fetch_rankings(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        order_by: &str,
        limit: Option<i64>,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let mut sql = Self::select_sql(order_by);
        if limit.is_some() {
            sql.push_str(" LIMIT $3");
        }
        let mut query = sqlx::query(&sql)
            .bind(class_id.to_hex())
            .bind(exam_id.to_hex());
        if let Some(limit) = limit {
            query = query.bind(limit.max(1));
        }
        let rows = query.fetch_all(&self.pool).await.map_err(Self::db_error)?;
        rows.into_iter().map(Self::result_from_row).collect()
    }
}
