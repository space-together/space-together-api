use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::{
    domain::{
        score::Score,
        student_term_result::{CategoryScore, StudentTermResult, SubjectResult},
    },
    errors::AppError,
    utils::object_id::ObjectId,
};

#[derive(Debug, Clone)]
struct SubjectMeta {
    name: String,
    credits: Option<i32>,
}

#[derive(Debug, Clone)]
struct CategoryMeta {
    name: String,
    weight_percentage: f64,
}

pub struct GpaCalculationService {
    pub pool: PgPool,
}

impl GpaCalculationService {
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

    fn score_from_row(row: PgRow) -> Result<Score, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Score {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            student_id: Self::parse_oid_opt(
                row.try_get("student_id").ok().flatten(),
                "student_id",
            )?,
            class_subject_id: Self::parse_oid_opt(
                row.try_get("class_subject_id").ok().flatten(),
                "class_subject_id",
            )?,
            exam_id: Self::parse_oid_opt(row.try_get("exam_id").ok().flatten(), "exam_id")?,
            assessment_category_id: Self::parse_oid_opt(
                row.try_get("assessment_category_id").ok().flatten(),
                "assessment_category_id",
            )?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            score: row.try_get("score").map_err(Self::db_error)?,
            max_score: row.try_get("max_score").map_err(Self::db_error)?,
            percentage: row.try_get("percentage").map_err(Self::db_error)?,
            remarks: row.try_get("remarks").ok().flatten(),
            entered_by: Self::parse_oid_opt(
                row.try_get("entered_by").ok().flatten(),
                "entered_by",
            )?,
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
            is_deleted: row
                .try_get::<Option<DateTime<Utc>>, _>("deleted_at")
                .ok()
                .flatten()
                .is_some(),
        })
    }

    async fn hydrate_result_subjects(
        &self,
        result: &mut StudentTermResult,
    ) -> Result<(), AppError> {
        let Some(result_id) = result.id else {
            return Ok(());
        };

        let subject_rows = sqlx::query(
            r#"
            SELECT id, class_subject_id, subject_name, weighted_score, percentage, grade, credits
            FROM student_term_subject_results
            WHERE result_id = $1
            ORDER BY position ASC, subject_name ASC
            "#,
        )
        .bind(result_id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut subjects = Vec::with_capacity(subject_rows.len());
        for subject_row in subject_rows {
            let subject_result_id: String = subject_row.try_get("id").map_err(Self::db_error)?;
            let category_rows = sqlx::query(
                r#"
                SELECT assessment_category_id, category_name, score, max_score, weight_percentage
                FROM student_term_category_scores
                WHERE subject_result_id = $1
                ORDER BY position ASC, category_name ASC
                "#,
            )
            .bind(&subject_result_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

            let category_scores = category_rows
                .into_iter()
                .map(|row| {
                    Ok(CategoryScore {
                        assessment_category_id: Self::parse_oid_opt(
                            row.try_get("assessment_category_id").ok().flatten(),
                            "assessment_category_id",
                        )?,
                        category_name: row.try_get("category_name").map_err(Self::db_error)?,
                        score: row.try_get("score").map_err(Self::db_error)?,
                        max_score: row.try_get("max_score").map_err(Self::db_error)?,
                        weight_percentage: row
                            .try_get("weight_percentage")
                            .map_err(Self::db_error)?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;

            subjects.push(SubjectResult {
                class_subject_id: Self::parse_oid_opt(
                    subject_row.try_get("class_subject_id").ok().flatten(),
                    "class_subject_id",
                )?,
                subject_name: subject_row
                    .try_get("subject_name")
                    .map_err(Self::db_error)?,
                category_scores,
                weighted_score: subject_row
                    .try_get("weighted_score")
                    .map_err(Self::db_error)?,
                percentage: subject_row.try_get("percentage").map_err(Self::db_error)?,
                grade: subject_row.try_get("grade").map_err(Self::db_error)?,
                credits: subject_row.try_get("credits").ok().flatten(),
            });
        }
        result.subject_results = subjects;
        Ok(())
    }

    pub async fn calculate_student_result(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
        class_id: &ObjectId,
        education_year_id: &ObjectId,
        school_id: &ObjectId,
        term_id: Option<String>,
    ) -> Result<StudentTermResult, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, school_id, student_id, class_subject_id, exam_id,
                   assessment_category_id, education_year_id,
                   score::DOUBLE PRECISION AS score,
                   max_score::DOUBLE PRECISION AS max_score,
                   percentage::DOUBLE PRECISION AS percentage,
                   remarks, COALESCE(entered_by, recorded_by) AS entered_by,
                   created_at, updated_at, deleted_at
            FROM scores
            WHERE student_id = $1 AND exam_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(student_id.to_hex())
        .bind(exam_id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let scores = rows
            .into_iter()
            .map(Self::score_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        if scores.is_empty() {
            return Err(AppError {
                message: "No scores found for this student and exam".to_string(),
            });
        }

        let mut subject_scores: HashMap<ObjectId, Vec<Score>> = HashMap::new();
        for score in scores {
            if let Some(subject_id) = score.class_subject_id {
                subject_scores.entry(subject_id).or_default().push(score);
            }
        }

        let mut subject_results = Vec::new();
        let mut total_weighted_score = 0.0;
        let mut total_credits = 0;

        for (subject_id, scores) in subject_scores {
            let subject_result = self
                .calculate_subject_result(&subject_id, scores, education_year_id)
                .await?;
            total_weighted_score += subject_result.weighted_score;
            if let Some(credits) = subject_result.credits {
                total_credits += credits;
            }
            subject_results.push(subject_result);
        }

        let subject_count = subject_results.len() as f64;
        let average_percentage = if subject_count > 0.0 {
            total_weighted_score / subject_count
        } else {
            0.0
        };
        let gpa = average_percentage / 25.0;
        let grade = self
            .calculate_grade_from_active_scale(school_id, education_year_id, average_percentage)
            .await?;

        let result = StudentTermResult {
            id: None,
            school_id: Some(*school_id),
            student_id: Some(*student_id),
            class_id: Some(*class_id),
            education_year_id: Some(*education_year_id),
            term_id,
            exam_id: Some(*exam_id),
            subject_results,
            total_score: total_weighted_score,
            total_max_score: subject_count * 100.0,
            average_percentage,
            gpa,
            total_credits: if total_credits > 0 {
                Some(total_credits)
            } else {
                None
            },
            grade,
            rank_in_class: None,
            total_students: None,
            calculated_at: Some(Utc::now()),
            is_finalized: false,
        };

        self.save_result(&result).await
    }

    async fn calculate_subject_result(
        &self,
        subject_id: &ObjectId,
        scores: Vec<Score>,
        education_year_id: &ObjectId,
    ) -> Result<SubjectResult, AppError> {
        let subject = self.fetch_subject_meta(subject_id).await?;
        let categories = self
            .fetch_categories_for_subject(subject_id, education_year_id)
            .await?;
        let mut category_scores = Vec::new();
        let mut weighted_total = 0.0;

        for score in scores {
            if let Some(category_id) = score.assessment_category_id {
                if let Some(category) = categories.get(&category_id) {
                    let weighted_contribution =
                        score.percentage * (category.weight_percentage / 100.0);
                    weighted_total += weighted_contribution;
                    category_scores.push(CategoryScore {
                        assessment_category_id: Some(category_id),
                        category_name: category.name.clone(),
                        score: score.score,
                        max_score: score.max_score,
                        weight_percentage: category.weight_percentage,
                    });
                }
            }
        }

        Ok(SubjectResult {
            class_subject_id: Some(*subject_id),
            subject_name: subject.name,
            category_scores,
            weighted_score: weighted_total,
            percentage: weighted_total,
            grade: self.determine_grade(weighted_total),
            credits: subject.credits,
        })
    }

    async fn fetch_subject_meta(&self, subject_id: &ObjectId) -> Result<SubjectMeta, AppError> {
        let row = sqlx::query(
            "SELECT name, credits FROM class_subjects WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(subject_id.to_hex())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?
        .ok_or_else(|| AppError {
            message: "Subject not found".to_string(),
        })?;

        Ok(SubjectMeta {
            name: row.try_get("name").map_err(Self::db_error)?,
            credits: row.try_get("credits").ok().flatten(),
        })
    }

    async fn fetch_categories_for_subject(
        &self,
        subject_id: &ObjectId,
        education_year_id: &ObjectId,
    ) -> Result<HashMap<ObjectId, CategoryMeta>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, COALESCE(weight_percentage, weight, 0)::DOUBLE PRECISION AS weight_percentage
            FROM assessment_categories
            WHERE class_subject_id = $1 AND education_year_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(subject_id.to_hex())
        .bind(education_year_id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut categories = HashMap::new();
        for row in rows {
            let id: String = row.try_get("id").map_err(Self::db_error)?;
            categories.insert(
                Self::parse_oid(&id, "assessment_category_id")?,
                CategoryMeta {
                    name: row.try_get("name").map_err(Self::db_error)?,
                    weight_percentage: row.try_get("weight_percentage").map_err(Self::db_error)?,
                },
            );
        }
        Ok(categories)
    }

    fn determine_grade(&self, percentage: f64) -> String {
        if percentage >= 90.0 {
            "A".to_string()
        } else if percentage >= 80.0 {
            "B".to_string()
        } else if percentage >= 70.0 {
            "C".to_string()
        } else if percentage >= 60.0 {
            "D".to_string()
        } else if percentage >= 50.0 {
            "E".to_string()
        } else {
            "F".to_string()
        }
    }

    async fn calculate_grade_from_active_scale(
        &self,
        school_id: &ObjectId,
        education_year_id: &ObjectId,
        percentage: f64,
    ) -> Result<String, AppError> {
        let row = sqlx::query(
            r#"
            SELECT gb.grade
            FROM grading_scales gs
            JOIN grading_scale_boundaries gb ON gb.grading_scale_id = gs.id
            WHERE gs.school_id = $1
              AND gs.education_year_id = $2
              AND gs.is_active = true
              AND gs.deleted_at IS NULL
              AND $3 BETWEEN gb.min_score AND gb.max_score
            ORDER BY gb.position ASC
            LIMIT 1
            "#,
        )
        .bind(school_id.to_hex())
        .bind(education_year_id.to_hex())
        .bind(percentage)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(row
            .and_then(|row| row.try_get::<String, _>("grade").ok())
            .unwrap_or_else(|| "N/A".to_string()))
    }

    async fn save_result(&self, result: &StudentTermResult) -> Result<StudentTermResult, AppError> {
        let school_id = result.school_id.ok_or_else(|| AppError {
            message: "Result has no school_id".into(),
        })?;
        let student_id = result.student_id.ok_or_else(|| AppError {
            message: "Result has no student_id".into(),
        })?;
        let class_id = result.class_id.ok_or_else(|| AppError {
            message: "Result has no class_id".into(),
        })?;
        let education_year_id = result.education_year_id.ok_or_else(|| AppError {
            message: "Result has no education_year_id".into(),
        })?;
        let exam_id = result.exam_id.ok_or_else(|| AppError {
            message: "Result has no exam_id".into(),
        })?;

        let existing_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM student_term_results WHERE student_id = $1 AND exam_id = $2 AND deleted_at IS NULL LIMIT 1",
        )
        .bind(student_id.to_hex())
        .bind(exam_id.to_hex())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        let result_id = existing_id.unwrap_or_else(Self::new_id);
        let term_id = result
            .term_id
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let academic_year = education_year_id.to_hex();

        sqlx::query(
            r#"
            INSERT INTO student_term_results (
              id, school_id, student_id, class_id, education_year_id, exam_id,
              term_id, term, academic_year, total_score, total_max_score,
              average_score, average_percentage, gpa, total_credits, grade,
              calculated_at, is_finalized, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, $10, $11, $11, $12, $13, $14, $15, $16, now())
            ON CONFLICT (id) DO UPDATE SET
              school_id = EXCLUDED.school_id,
              student_id = EXCLUDED.student_id,
              class_id = EXCLUDED.class_id,
              education_year_id = EXCLUDED.education_year_id,
              exam_id = EXCLUDED.exam_id,
              term_id = EXCLUDED.term_id,
              term = EXCLUDED.term,
              academic_year = EXCLUDED.academic_year,
              total_score = EXCLUDED.total_score,
              total_max_score = EXCLUDED.total_max_score,
              average_score = EXCLUDED.average_score,
              average_percentage = EXCLUDED.average_percentage,
              gpa = EXCLUDED.gpa,
              total_credits = EXCLUDED.total_credits,
              grade = EXCLUDED.grade,
              calculated_at = EXCLUDED.calculated_at,
              is_finalized = EXCLUDED.is_finalized,
              updated_at = now()
            "#,
        )
        .bind(&result_id)
        .bind(school_id.to_hex())
        .bind(student_id.to_hex())
        .bind(class_id.to_hex())
        .bind(education_year_id.to_hex())
        .bind(exam_id.to_hex())
        .bind(&term_id)
        .bind(&academic_year)
        .bind(result.total_score)
        .bind(result.total_max_score)
        .bind(result.average_percentage)
        .bind(result.gpa)
        .bind(result.total_credits)
        .bind(&result.grade)
        .bind(result.calculated_at.unwrap_or_else(Utc::now))
        .bind(result.is_finalized)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.replace_result_breakdowns(&result_id, &result.subject_results)
            .await?;
        self.get_result_by_id(&result_id).await
    }

    async fn replace_result_breakdowns(
        &self,
        result_id: &str,
        subject_results: &[SubjectResult],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM student_term_subject_results WHERE result_id = $1")
            .bind(result_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (subject_position, subject) in subject_results.iter().enumerate() {
            let subject_result_id = Self::new_id();
            sqlx::query(
                r#"
                INSERT INTO student_term_subject_results (
                  id, result_id, class_subject_id, subject_name, weighted_score,
                  percentage, grade, credits, position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(&subject_result_id)
            .bind(result_id)
            .bind(subject.class_subject_id.as_ref().map(|id| id.to_hex()))
            .bind(&subject.subject_name)
            .bind(subject.weighted_score)
            .bind(subject.percentage)
            .bind(&subject.grade)
            .bind(subject.credits)
            .bind(subject_position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

            for (category_position, category) in subject.category_scores.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO student_term_category_scores (
                      id, subject_result_id, assessment_category_id, category_name,
                      score, max_score, weight_percentage, position
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                )
                .bind(Self::new_id())
                .bind(&subject_result_id)
                .bind(
                    category
                        .assessment_category_id
                        .as_ref()
                        .map(|id| id.to_hex()),
                )
                .bind(&category.category_name)
                .bind(category.score)
                .bind(category.max_score)
                .bind(category.weight_percentage)
                .bind(category_position as i32)
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
            }
        }

        Ok(())
    }

    async fn get_result_by_id(&self, result_id: &str) -> Result<StudentTermResult, AppError> {
        let row = sqlx::query(Self::result_select_sql())
            .bind(result_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .ok_or_else(|| AppError {
                message: "Result not found".into(),
            })?;
        let mut result = Self::result_from_row(row)?;
        self.hydrate_result_subjects(&mut result).await?;
        Ok(result)
    }

    fn result_select_sql() -> &'static str {
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
        WHERE id = $1 AND deleted_at IS NULL
        "#
    }

    pub async fn calculate_class_results(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        education_year_id: &ObjectId,
        school_id: &ObjectId,
        term_id: Option<String>,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT student_id
            FROM student_school_enrollments
            WHERE class_id = $1 AND school_id = $2 AND is_active = true AND deleted_at IS NULL
            ORDER BY updated_at DESC
            "#,
        )
        .bind(class_id.to_hex())
        .bind(school_id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut results = Vec::new();
        for row in rows {
            let student_id: String = row.try_get("student_id").map_err(Self::db_error)?;
            let student_id = Self::parse_oid(&student_id, "student_id")?;
            match self
                .calculate_student_result(
                    &student_id,
                    exam_id,
                    class_id,
                    education_year_id,
                    school_id,
                    term_id.clone(),
                )
                .await
            {
                Ok(result) => results.push(result),
                Err(error) => {
                    eprintln!(
                        "Failed to calculate result for student {}: {}",
                        student_id, error.message
                    );
                }
            }
        }

        Ok(results)
    }

    pub async fn get_student_result(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Option<StudentTermResult>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM student_term_results
            WHERE student_id = $1 AND exam_id = $2 AND deleted_at IS NULL
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(student_id.to_hex())
        .bind(exam_id.to_hex())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;

        match row {
            Some(row) => {
                let id: String = row.try_get("id").map_err(Self::db_error)?;
                Ok(Some(self.get_result_by_id(&id).await?))
            }
            None => Ok(None),
        }
    }
}
