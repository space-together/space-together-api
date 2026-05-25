use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::analytics::{
        AttendanceRate, EnrollmentTrend, FeeCollectionSummary, PassFailDistribution,
        TeacherWorkload,
    },
    errors::AppError,
    models::id_model::IdType,
};

pub struct AnalyticsService {
    pub pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(error: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", error),
        }
    }

    fn id_to_string(id: &IdType) -> Result<String, AppError> {
        Ok(IdType::to_object_id(id)?.to_hex())
    }

    pub async fn get_enrollment_trends(
        &self,
        school_id: &IdType,
        year: Option<i32>,
    ) -> Result<Vec<EnrollmentTrend>, AppError> {
        let school_id = Self::id_to_string(school_id)?;
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT to_char(created_at, 'YYYY-MM') AS month, count(*)::BIGINT AS total
            FROM student_school_enrollments
            WHERE school_id =
            "#,
        );
        query.push_bind(school_id).push(" AND deleted_at IS NULL");

        if let Some(year) = year {
            query
                .push(" AND EXTRACT(YEAR FROM created_at)::INTEGER = ")
                .push_bind(year);
        }

        query.push(" GROUP BY month ORDER BY month ASC");

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(EnrollmentTrend {
                    month: row.try_get("month").map_err(Self::db_error)?,
                    total: row.try_get("total").map_err(Self::db_error)?,
                })
            })
            .collect()
    }

    pub async fn get_attendance_rate(
        &self,
        school_id: &IdType,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<AttendanceRate, AppError> {
        let school_id = Self::id_to_string(school_id)?;
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT count(*)::BIGINT AS total_records,
                   count(*) FILTER (WHERE lower(status) = 'present')::BIGINT AS present_count
            FROM attendance
            WHERE school_id =
            "#,
        );
        query.push_bind(school_id).push(" AND deleted_at IS NULL");

        if let Some(from) = from {
            query.push(" AND date >= ").push_bind(from.date_naive());
        }

        if let Some(to) = to {
            query.push(" AND date <= ").push_bind(to.date_naive());
        }

        let row = query
            .build()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let total_records: i64 = row.try_get("total_records").map_err(Self::db_error)?;
        let present_count: i64 = row.try_get("present_count").map_err(Self::db_error)?;
        let attendance_rate = if total_records > 0 {
            (present_count as f64 / total_records as f64) * 100.0
        } else {
            0.0
        };

        Ok(AttendanceRate {
            attendance_rate,
            total_records,
            present_count,
        })
    }

    pub async fn get_pass_fail_distribution(
        &self,
        school_id: &IdType,
        passing_mark: Option<f64>,
    ) -> Result<PassFailDistribution, AppError> {
        let school_id = Self::id_to_string(school_id)?;
        let passing_threshold = passing_mark.unwrap_or(50.0);

        let row = sqlx::query(
            r#"
            SELECT count(*) FILTER (WHERE percentage >= $2)::BIGINT AS pass,
                   count(*) FILTER (WHERE percentage < $2)::BIGINT AS fail,
                   count(*)::BIGINT AS total
            FROM scores
            WHERE school_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(school_id)
        .bind(passing_threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let pass: i64 = row.try_get("pass").map_err(Self::db_error)?;
        let fail: i64 = row.try_get("fail").map_err(Self::db_error)?;
        let total: i64 = row.try_get("total").map_err(Self::db_error)?;
        let pass_rate = if total > 0 {
            (pass as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(PassFailDistribution {
            pass,
            fail,
            total,
            pass_rate,
        })
    }

    pub async fn get_fee_summary(
        &self,
        school_id: &IdType,
    ) -> Result<FeeCollectionSummary, AppError> {
        let school_id = Self::id_to_string(school_id)?;
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount), 0)::DOUBLE PRECISION AS total_expected,
                   COALESCE(SUM(amount) FILTER (WHERE lower(status) = 'paid'), 0)::DOUBLE PRECISION AS total_collected
            FROM finance_records
            WHERE school_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(school_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let total_expected: f64 = row.try_get("total_expected").map_err(Self::db_error)?;
        let total_collected: f64 = row.try_get("total_collected").map_err(Self::db_error)?;
        let total_outstanding = total_expected - total_collected;
        let collection_rate = if total_expected > 0.0 {
            (total_collected / total_expected) * 100.0
        } else {
            0.0
        };

        Ok(FeeCollectionSummary {
            total_expected,
            total_collected,
            total_outstanding,
            collection_rate,
        })
    }

    pub async fn get_teacher_workload(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWorkload>, AppError> {
        let school_id = Self::id_to_string(school_id)?;
        let rows = sqlx::query(
            r#"
            SELECT t.id AS teacher_id,
                   t.name AS teacher_name,
                   count(DISTINCT cs.class_id)::BIGINT AS classes,
                   count(DISTINCT cs.id)::BIGINT AS subjects,
                   count(DISTINCT enrollments.student_id)::BIGINT AS total_students
            FROM teachers t
            LEFT JOIN class_subjects cs
              ON cs.school_id = t.school_id
             AND cs.deleted_at IS NULL
             AND COALESCE(cs.disable, false) = false
             AND (cs.teacher_id = t.id OR (cs.teacher_id IS NULL AND cs.teacher_user_id = t.user_id))
            LEFT JOIN student_school_enrollments enrollments
              ON enrollments.school_id = t.school_id
             AND enrollments.class_id = cs.class_id
             AND enrollments.is_active = true
             AND enrollments.deleted_at IS NULL
            WHERE t.school_id = $1
              AND t.is_active = true
              AND t.deleted_at IS NULL
            GROUP BY t.id, t.name
            ORDER BY total_students DESC, teacher_name ASC
            "#,
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(TeacherWorkload {
                    teacher_id: row.try_get("teacher_id").map_err(Self::db_error)?,
                    teacher_name: row.try_get("teacher_name").map_err(Self::db_error)?,
                    classes: row.try_get("classes").map_err(Self::db_error)?,
                    subjects: row.try_get("subjects").map_err(Self::db_error)?,
                    total_students: row.try_get("total_students").map_err(Self::db_error)?,
                })
            })
            .collect()
    }
}
