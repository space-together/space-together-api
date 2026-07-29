use crate::domain::{
    common_details::{
        Address, Age, Image, Language, SocialMedia, StudyStyle, SubjectCategory, UserRole,
    },
    user::{PaginatedUsers, UpdateUserDto, User, UserStats},
};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::services::user_service::normalize_user_ids;
use crate::utils::object_id::ObjectId;
use chrono::{Duration, Utc, Weekday};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};
use std::collections::HashMap;

pub struct UserRepo {
    pub pool: PgPool,
}

impl UserRepo {
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

    fn address_from_row(row: &PgRow) -> Result<Option<Address>, AppError> {
        let country: Option<String> = row.try_get("address_country").map_err(Self::db_error)?;
        Ok(country.map(|country| Address {
            country,
            province: row.try_get("address_province").ok().flatten(),
            district: row.try_get("address_district").ok().flatten(),
            sector: row.try_get("address_sector").ok().flatten(),
            cell: row.try_get("address_cell").ok().flatten(),
            village: row.try_get("address_village").ok().flatten(),
            state: row.try_get("address_state").ok().flatten(),
            street: row.try_get("address_street").ok().flatten(),
            city: row.try_get("address_city").ok().flatten(),
            postal_code: row.try_get("address_postal_code").ok().flatten(),
            google_map_url: row.try_get("address_google_map_url").ok().flatten(),
        }))
    }

    async fn fetch_school_ids(&self, user_id: &str) -> Result<Option<Vec<ObjectId>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT school_id
            FROM school_memberships
            WHERE user_id = $1 AND status = 'active'
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("school_id").map_err(Self::db_error)?;
                Self::parse_oid(&id, "school_id")
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn fetch_accessible_classes(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<ObjectId>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT class_id
            FROM school_user_accessible_classes
            WHERE user_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("class_id").map_err(Self::db_error)?;
                Self::parse_oid(&id, "class_id")
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn fetch_background_images(&self, user_id: &str) -> Result<Option<Vec<Image>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT image_id, url
            FROM user_background_images
            WHERE user_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let images = rows
            .into_iter()
            .map(|row| {
                Ok(Image {
                    id: row.try_get("image_id").map_err(Self::db_error)?,
                    url: row.try_get("url").map_err(Self::db_error)?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((!images.is_empty()).then_some(images))
    }

    async fn fetch_social_media(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<SocialMedia>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT platform, url
            FROM user_social_media
            WHERE user_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let items = rows
            .into_iter()
            .map(|row| {
                Ok(SocialMedia {
                    platform: row.try_get("platform").map_err(Self::db_error)?,
                    url: row.try_get("url").map_err(Self::db_error)?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((!items.is_empty()).then_some(items))
    }

    async fn fetch_profile_values(
        &self,
        user_id: &str,
    ) -> Result<HashMap<String, Vec<String>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT value_type, value
            FROM user_profile_values
            WHERE user_id = $1
            ORDER BY value_type ASC, position ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let mut values: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let value_type: String = row.try_get("value_type").map_err(Self::db_error)?;
            let value: String = row.try_get("value").map_err(Self::db_error)?;
            values.entry(value_type).or_default().push(value);
        }

        Ok(values)
    }

    fn values_as_strings(values: &HashMap<String, Vec<String>>, key: &str) -> Option<Vec<String>> {
        values.get(key).filter(|items| !items.is_empty()).cloned()
    }

    fn values_as_enums<T: DeserializeOwned>(
        values: &HashMap<String, Vec<String>>,
        key: &str,
    ) -> Option<Vec<T>> {
        let parsed = values.get(key).map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    serde_json::from_value(serde_json::Value::String(item.clone())).ok()
                })
                .collect::<Vec<T>>()
        })?;

        (!parsed.is_empty()).then_some(parsed)
    }

    fn values_as_ids(
        values: &HashMap<String, Vec<String>>,
        key: &str,
    ) -> Result<Option<Vec<ObjectId>>, AppError> {
        let Some(items) = values.get(key) else {
            return Ok(None);
        };

        let ids = items
            .iter()
            .map(|item| Self::parse_oid(item, key))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn user_from_row(&self, row: PgRow) -> Result<User, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let values = self.fetch_profile_values(&id).await?;

        Ok(User {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            email: row.try_get("email").map_err(Self::db_error)?,
            username: row.try_get("username").ok().flatten(),
            password_hash: row.try_get("password_hash").ok().flatten(),
            role: Self::enum_from_string(row.try_get("role").ok().flatten()),
            image_id: row.try_get("image_id").ok().flatten(),
            image: row.try_get("image").ok().flatten(),
            background_images: self.fetch_background_images(&id).await?,
            bio: row.try_get("bio").ok().flatten(),
            disable: row.try_get("disabled").ok().flatten(),
            phone: row.try_get("phone").ok().flatten(),
            address: Self::address_from_row(&row)?,
            social_media: self.fetch_social_media(&id).await?,
            preferred_communication_method: Self::values_as_enums(
                &values,
                "preferred_communication_method",
            ),
            gender: Self::enum_from_string(row.try_get("gender").ok().flatten()),
            age: match (
                row.try_get("age_year").ok().flatten(),
                row.try_get("age_month").ok().flatten(),
                row.try_get("age_day").ok().flatten(),
            ) {
                (Some(year), Some(month), Some(day)) => Some(Age { year, month, day }),
                _ => None,
            },
            languages_spoken: Self::values_as_enums::<Language>(&values, "languages_spoken"),
            hobbies_interests: Self::values_as_strings(&values, "hobbies_interests"),
            dream_career: row.try_get("dream_career").ok().flatten(),
            special_skills: Self::values_as_strings(&values, "special_skills"),
            health_or_learning_notes: row.try_get("health_or_learning_notes").ok().flatten(),
            current_school_id: Self::parse_oid_opt(
                row.try_get("current_school_id").ok().flatten(),
                "current_school_id",
            )?,
            schools: self.fetch_school_ids(&id).await?,
            accessible_classes: self.fetch_accessible_classes(&id).await?,
            favorite_subjects_category: Self::values_as_enums::<SubjectCategory>(
                &values,
                "favorite_subjects_category",
            ),
            preferred_study_styles: Self::values_as_enums::<StudyStyle>(
                &values,
                "preferred_study_styles",
            ),
            guardian_info: None,
            special_support_needed: Self::values_as_enums(&values, "special_support_needed"),
            learning_challenges: Self::values_as_enums(&values, "learning_challenges"),
            teaching_level: Self::values_as_ids(&values, "teaching_level")?,
            employment_type: Self::enum_from_string(row.try_get("employment_type").ok().flatten()),
            teaching_start_date: row.try_get("teaching_start_date").ok().flatten(),
            years_of_experience: row.try_get("years_of_experience").ok().flatten(),
            education_level: Self::enum_from_string(row.try_get("education_level").ok().flatten()),
            certifications_trainings: None,
            preferred_age_group: Self::enum_from_string(
                row.try_get("preferred_age_group").ok().flatten(),
            ),
            professional_goals: Self::values_as_enums(&values, "professional_goals"),
            availability_schedule: None,
            department: Self::enum_from_string(row.try_get("department").ok().flatten()),
            job_title: Self::enum_from_string(row.try_get("job_title").ok().flatten()),
            teaching_style: Self::values_as_enums(&values, "teaching_style"),
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
        })
    }

    pub(crate) async fn user_from_sql_row(&self, row: PgRow) -> Result<User, AppError> {
        self.user_from_row(row).await
    }

    async fn find_one_sql(&self, sql: &str, value: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query(sql)
            .bind(value)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.user_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.find_one_sql(
            "SELECT * FROM users WHERE lower(email) = lower($1) AND deleted_at IS NULL",
            email,
        )
        .await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.find_one_sql(
            "SELECT * FROM users WHERE lower(username) = lower($1) AND deleted_at IS NULL",
            username,
        )
        .await
    }

    pub async fn find_by_login(&self, login: &str) -> Result<Option<User>, AppError> {
        self.find_one_sql(
            r#"
            SELECT *
            FROM users
            WHERE deleted_at IS NULL
              AND (lower(email) = lower($1) OR lower(username) = lower($1))
            "#,
            login,
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<User>, AppError> {
        let id = Self::id_to_string(id)?;
        self.find_one_sql(
            "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL",
            &id,
        )
        .await
    }

    pub async fn find_one(&self, id: Option<&IdType>) -> Result<User, AppError> {
        let id = id.ok_or_else(|| AppError {
            message: "User ID is required".into(),
        })?;
        self.find_by_id(id).await?.ok_or(AppError {
            message: "User not found".into(),
        })
    }

    async fn sync_background_images(
        &self,
        user_id: &str,
        images: Option<&Vec<Image>>,
    ) -> Result<(), AppError> {
        let Some(images) = images else {
            return Ok(());
        };

        sqlx::query("DELETE FROM user_background_images WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, image) in images.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO user_background_images (id, user_id, image_id, url, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(user_id)
            .bind(&image.id)
            .bind(&image.url)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn sync_social_media(
        &self,
        user_id: &str,
        social_media: Option<&Vec<SocialMedia>>,
    ) -> Result<(), AppError> {
        let Some(social_media) = social_media else {
            return Ok(());
        };

        sqlx::query("DELETE FROM user_social_media WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, item) in social_media.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO user_social_media (id, user_id, platform, url, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(user_id)
            .bind(&item.platform)
            .bind(&item.url)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn replace_profile_values(
        &self,
        user_id: &str,
        value_type: &str,
        values: &[String],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM user_profile_values WHERE user_id = $1 AND value_type = $2")
            .bind(user_id)
            .bind(value_type)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, value) in values.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO user_profile_values (id, user_id, value_type, value, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(user_id)
            .bind(value_type)
            .bind(value)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    fn enum_vec_to_strings<T: Serialize>(items: &[T]) -> Vec<String> {
        items.iter().filter_map(Self::enum_to_string).collect()
    }

    async fn sync_user_children(&self, user_id: &str, user: &User) -> Result<(), AppError> {
        self.sync_background_images(user_id, user.background_images.as_ref())
            .await?;
        self.sync_social_media(user_id, user.social_media.as_ref())
            .await?;

        if let Some(values) = &user.languages_spoken {
            self.replace_profile_values(
                user_id,
                "languages_spoken",
                &Self::enum_vec_to_strings(values),
            )
            .await?;
        }
        if let Some(values) = &user.hobbies_interests {
            self.replace_profile_values(user_id, "hobbies_interests", values)
                .await?;
        }
        if let Some(values) = &user.special_skills {
            self.replace_profile_values(user_id, "special_skills", values)
                .await?;
        }
        if let Some(values) = &user.favorite_subjects_category {
            self.replace_profile_values(
                user_id,
                "favorite_subjects_category",
                &Self::enum_vec_to_strings(values),
            )
            .await?;
        }
        if let Some(values) = &user.preferred_study_styles {
            self.replace_profile_values(
                user_id,
                "preferred_study_styles",
                &Self::enum_vec_to_strings(values),
            )
            .await?;
        }
        if let Some(values) = &user.teaching_level {
            let ids = values.iter().map(|id| id.to_hex()).collect::<Vec<_>>();
            self.replace_profile_values(user_id, "teaching_level", &ids)
                .await?;
        }

        Ok(())
    }

    pub async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(())
    }

    pub async fn find_school_user_id(
        &self,
        user_id: &str,
        school_id: &str,
    ) -> Result<Option<String>, AppError> {
        sqlx::query_scalar(
            r#"
            SELECT school_user_id
            FROM school_memberships
            WHERE user_id = $1 AND school_id = $2 AND status = 'active'
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(school_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)
    }

    pub async fn insert_user(&self, user: &mut User) -> Result<User, AppError> {
        self.ensure_indexes().await?;

        let mut user_to_insert = normalize_user_ids(user.clone())?;
        let id = user_to_insert
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);

        let now = Utc::now();
        user_to_insert.created_at = Some(user_to_insert.created_at.unwrap_or(now));
        user_to_insert.updated_at = Some(user_to_insert.updated_at.unwrap_or(now));

        if user_to_insert.availability_schedule.is_none() {
            use crate::domain::common_details::{DailyAvailability, TimeRange};
            user_to_insert.availability_schedule = Some(
                vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ]
                .into_iter()
                .map(|day| DailyAvailability {
                    day,
                    time_range: TimeRange::new("09:00", "17:00"),
                })
                .collect(),
            );
        }

        let address = user_to_insert.address.as_ref();
        let age = user_to_insert.age.as_ref();

        sqlx::query(
            r#"
            INSERT INTO users (
              id, name, email, username, password_hash, role, image, image_id, phone, gender,
              disabled, current_school_id, bio, age_year, age_month, age_day, dream_career,
              health_or_learning_notes, employment_type, teaching_start_date, years_of_experience,
              education_level, preferred_age_group, department, job_title, address_country,
              address_province, address_district, address_sector, address_cell, address_village,
              address_state, address_street, address_city, address_postal_code, address_google_map_url,
              created_at, updated_at
            )
            VALUES (
              $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
              $11, $12, $13, $14, $15, $16, $17,
              $18, $19, $20, $21,
              $22, $23, $24, $25, $26,
              $27, $28, $29, $30, $31,
              $32, $33, $34, $35, $36,
              $37, $38
            )
            "#,
        )
        .bind(&id)
        .bind(&user_to_insert.name)
        .bind(&user_to_insert.email)
        .bind(&user_to_insert.username)
        .bind(&user_to_insert.password_hash)
        .bind(user_to_insert.role.as_ref().map(UserRole::to_string))
        .bind(&user_to_insert.image)
        .bind(&user_to_insert.image_id)
        .bind(&user_to_insert.phone)
        .bind(user_to_insert.gender.as_ref().map(ToString::to_string))
        .bind(user_to_insert.disable.unwrap_or(false))
        .bind(user_to_insert.current_school_id.as_ref().map(|id| id.to_hex()))
        .bind(&user_to_insert.bio)
        .bind(age.map(|age| age.year))
        .bind(age.map(|age| age.month))
        .bind(age.map(|age| age.day))
        .bind(&user_to_insert.dream_career)
        .bind(&user_to_insert.health_or_learning_notes)
        .bind(user_to_insert.employment_type.as_ref().and_then(Self::enum_to_string))
        .bind(user_to_insert.teaching_start_date)
        .bind(user_to_insert.years_of_experience)
        .bind(user_to_insert.education_level.as_ref().and_then(Self::enum_to_string))
        .bind(user_to_insert.preferred_age_group.as_ref().and_then(Self::enum_to_string))
        .bind(user_to_insert.department.as_ref().and_then(Self::enum_to_string))
        .bind(user_to_insert.job_title.as_ref().and_then(Self::enum_to_string))
        .bind(address.map(|address| address.country.clone()))
        .bind(address.and_then(|address| address.province.clone()))
        .bind(address.and_then(|address| address.district.clone()))
        .bind(address.and_then(|address| address.sector.clone()))
        .bind(address.and_then(|address| address.cell.clone()))
        .bind(address.and_then(|address| address.village.clone()))
        .bind(address.and_then(|address| address.state.clone()))
        .bind(address.and_then(|address| address.street.clone()))
        .bind(address.and_then(|address| address.city.clone()))
        .bind(address.and_then(|address| address.postal_code.clone()))
        .bind(address.and_then(|address| address.google_map_url.clone()))
        .bind(user_to_insert.created_at)
        .bind(user_to_insert.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_user_children(&id, &user_to_insert).await?;

        self.find_by_id(&IdType::from_string(id))
            .await?
            .ok_or(AppError {
                message: "User not found after insert".into(),
            })
    }

    pub async fn get_all_users(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        _extra_match: Option<()>,
    ) -> Result<PaginatedUsers, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search = filter.unwrap_or_default();
        let search_like = format!("%{}%", search.to_lowercase());

        let total: i64 = if search.is_empty() {
            sqlx::query_scalar("SELECT count(*) FROM users WHERE deleted_at IS NULL")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::db_error)?
        } else {
            sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM users
                WHERE deleted_at IS NULL
                  AND (
                    lower(name) LIKE $1 OR lower(email) LIKE $1 OR lower(coalesce(username, '')) LIKE $1
                    OR lower(coalesce(phone, '')) LIKE $1 OR lower(coalesce(role, '')) LIKE $1
                    OR lower(coalesce(gender, '')) LIKE $1 OR lower(coalesce(bio, '')) LIKE $1
                    OR lower(id) LIKE $1 OR lower(coalesce(current_school_id, '')) LIKE $1
                  )
                "#,
            )
            .bind(&search_like)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?
        };

        let rows = if search.is_empty() {
            sqlx::query("SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(skip)
                .fetch_all(&self.pool)
                .await
                .map_err(Self::db_error)?
        } else {
            sqlx::query(
                r#"
                SELECT *
                FROM users
                WHERE deleted_at IS NULL
                  AND (
                    lower(name) LIKE $1 OR lower(email) LIKE $1 OR lower(coalesce(username, '')) LIKE $1
                    OR lower(coalesce(phone, '')) LIKE $1 OR lower(coalesce(role, '')) LIKE $1
                    OR lower(coalesce(gender, '')) LIKE $1 OR lower(coalesce(bio, '')) LIKE $1
                    OR lower(id) LIKE $1 OR lower(coalesce(current_school_id, '')) LIKE $1
                  )
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&search_like)
            .bind(limit)
            .bind(skip)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?
        };

        let mut users = Vec::with_capacity(rows.len());
        for row in rows {
            users.push(self.user_from_row(row).await?);
        }

        Ok(PaginatedUsers {
            users,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update_user_fields(
        &self,
        id: &str,
        update_dto: &UpdateUserDto,
    ) -> Result<User, AppError> {
        Self::parse_oid(id, "id")?;

        let mut query = QueryBuilder::<Postgres>::new("UPDATE users SET updated_at = now()");
        let mut touched_scalar = false;

        macro_rules! set_col {
            ($field:ident, $column:literal) => {
                if let Some(value) = &update_dto.$field {
                    query.push(", ");
                    query.push($column);
                    query.push(" = ");
                    query.push_bind(value);
                    touched_scalar = true;
                }
            };
        }

        set_col!(name, "name");
        set_col!(email, "email");
        set_col!(username, "username");
        set_col!(password_hash, "password_hash");
        set_col!(image_id, "image_id");
        set_col!(image, "image");
        set_col!(phone, "phone");
        set_col!(bio, "bio");
        set_col!(dream_career, "dream_career");
        set_col!(health_or_learning_notes, "health_or_learning_notes");

        if let Some(role) = &update_dto.role {
            query.push(", role = ").push_bind(role.to_string());
            touched_scalar = true;
        }
        if let Some(disable) = update_dto.disable {
            query.push(", disabled = ").push_bind(disable);
            touched_scalar = true;
        }
        if let Some(gender) = &update_dto.gender {
            query.push(", gender = ").push_bind(gender.to_string());
            touched_scalar = true;
        }
        if let Some(current_school_id) = &update_dto.current_school_id {
            query
                .push(", current_school_id = ")
                .push_bind(current_school_id.to_hex());
            touched_scalar = true;
        }
        if let Some(age) = &update_dto.age {
            query
                .push(", age_year = ")
                .push_bind(age.year)
                .push(", age_month = ")
                .push_bind(age.month)
                .push(", age_day = ")
                .push_bind(age.day);
            touched_scalar = true;
        }
        if let Some(address) = &update_dto.address {
            query
                .push(", address_country = ")
                .push_bind(&address.country)
                .push(", address_province = ")
                .push_bind(&address.province)
                .push(", address_district = ")
                .push_bind(&address.district)
                .push(", address_sector = ")
                .push_bind(&address.sector)
                .push(", address_cell = ")
                .push_bind(&address.cell)
                .push(", address_village = ")
                .push_bind(&address.village)
                .push(", address_state = ")
                .push_bind(&address.state)
                .push(", address_street = ")
                .push_bind(&address.street)
                .push(", address_city = ")
                .push_bind(&address.city)
                .push(", address_postal_code = ")
                .push_bind(&address.postal_code)
                .push(", address_google_map_url = ")
                .push_bind(&address.google_map_url);
            touched_scalar = true;
        }

        query
            .push(" WHERE id = ")
            .push_bind(id)
            .push(" AND deleted_at IS NULL");
        let result = query
            .build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let has_child_updates = update_dto.background_images.is_some()
            || update_dto.social_media.is_some()
            || update_dto.languages_spoken.is_some()
            || update_dto.hobbies_interests.is_some()
            || update_dto.special_skills.is_some()
            || update_dto.favorite_subjects_category.is_some()
            || update_dto.preferred_study_styles.is_some()
            || update_dto.teaching_level.is_some();

        if result.rows_affected() == 0 {
            return Err(AppError {
                message: "User not found".into(),
            });
        }

        if !touched_scalar && !has_child_updates {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        if let Some(images) = &update_dto.background_images {
            self.sync_background_images(id, Some(images)).await?;
        }
        if let Some(social) = &update_dto.social_media {
            self.sync_social_media(id, Some(social)).await?;
        }
        if let Some(values) = &update_dto.languages_spoken {
            self.replace_profile_values(id, "languages_spoken", &Self::enum_vec_to_strings(values))
                .await?;
        }
        if let Some(values) = &update_dto.hobbies_interests {
            self.replace_profile_values(id, "hobbies_interests", values)
                .await?;
        }
        if let Some(values) = &update_dto.special_skills {
            self.replace_profile_values(id, "special_skills", values)
                .await?;
        }
        if let Some(values) = &update_dto.favorite_subjects_category {
            self.replace_profile_values(
                id,
                "favorite_subjects_category",
                &Self::enum_vec_to_strings(values),
            )
            .await?;
        }
        if let Some(values) = &update_dto.preferred_study_styles {
            self.replace_profile_values(
                id,
                "preferred_study_styles",
                &Self::enum_vec_to_strings(values),
            )
            .await?;
        }
        if let Some(values) = &update_dto.teaching_level {
            let ids = values.iter().map(|id| id.to_hex()).collect::<Vec<_>>();
            self.replace_profile_values(id, "teaching_level", &ids)
                .await?;
        }

        self.find_by_id(&IdType::from_string(id))
            .await?
            .ok_or(AppError {
                message: "User not found after update".into(),
            })
    }

    pub async fn update(&self, id: &IdType, update_dto: &UpdateUserDto) -> Result<User, AppError> {
        self.update_user_fields(&id.as_string(), update_dto).await
    }

    pub async fn delete_user(&self, id: &IdType) -> Result<(), AppError> {
        let id = Self::id_to_string(id)?;
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(())
    }

    pub async fn soft_delete(&self, id: &IdType, deleted_by: ObjectId) -> Result<User, AppError> {
        let id = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE users SET deleted_at = now(), disabled = true, updated_at = now() WHERE id = $1",
        )
        .bind(&id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let _ = deleted_by;
        self.find_by_id(&IdType::from_string(id))
            .await?
            .ok_or(AppError {
                message: "User not found after soft delete".into(),
            })
    }

    pub async fn restore(&self, id: &IdType) -> Result<User, AppError> {
        let id = Self::id_to_string(id)?;
        sqlx::query(
            "UPDATE users SET deleted_at = NULL, disabled = false, updated_at = now() WHERE id = $1",
        )
        .bind(&id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_by_id(&IdType::from_string(id))
            .await?
            .ok_or(AppError {
                message: "User not found after restore".into(),
            })
    }

    pub async fn get_user_stats(&self) -> Result<UserStats, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
              count(*)::bigint AS total,
              count(*) FILTER (WHERE gender = 'MALE')::bigint AS male,
              count(*) FILTER (WHERE gender = 'FEMALE')::bigint AS female,
              count(*) FILTER (WHERE gender = 'OTHER')::bigint AS other,
              count(*) FILTER (WHERE role = 'ADMIN')::bigint AS admins,
              count(*) FILTER (WHERE role = 'SCHOOLSTAFF')::bigint AS staff,
              count(*) FILTER (WHERE role = 'STUDENT')::bigint AS students,
              count(*) FILTER (WHERE role = 'TEACHER')::bigint AS teachers,
              count(*) FILTER (WHERE current_school_id IS NOT NULL)::bigint AS assigned_school,
              count(*) FILTER (WHERE current_school_id IS NULL)::bigint AS no_school,
              count(*) FILTER (WHERE created_at >= $1)::bigint AS recent_30_days
            FROM users
            WHERE deleted_at IS NULL
            "#,
        )
        .bind(Utc::now() - Duration::days(30))
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(UserStats {
            total: row.try_get("total").map_err(Self::db_error)?,
            male: row.try_get("male").map_err(Self::db_error)?,
            female: row.try_get("female").map_err(Self::db_error)?,
            other: row.try_get("other").map_err(Self::db_error)?,
            admins: row.try_get("admins").map_err(Self::db_error)?,
            staff: row.try_get("staff").map_err(Self::db_error)?,
            students: row.try_get("students").map_err(Self::db_error)?,
            teachers: row.try_get("teachers").map_err(Self::db_error)?,
            assigned_school: row.try_get("assigned_school").map_err(Self::db_error)?,
            no_school: row.try_get("no_school").map_err(Self::db_error)?,
            recent_30_days: row.try_get("recent_30_days").map_err(Self::db_error)?,
        })
    }

    pub async fn add_school_to_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_id = Self::id_to_string(user_id)?;
        let school_id = Self::id_to_string(school_id)?;
        let user = self
            .find_by_id(&IdType::from_string(&user_id))
            .await?
            .ok_or(AppError {
                message: "User not found".into(),
            })?;
        let member_type = user
            .role
            .as_ref()
            .map(UserRole::to_string)
            .unwrap_or_else(|| "STUDENT".to_string());

        sqlx::query(
            r#"
            INSERT INTO school_memberships (id, school_id, user_id, member_type, status, joined_at)
            VALUES ($1, $2, $3, $4, 'active', now())
            ON CONFLICT (school_id, user_id, member_type)
            DO UPDATE SET status = 'active', ended_at = NULL, updated_at = now()
            "#,
        )
        .bind(Self::new_id())
        .bind(&school_id)
        .bind(&user_id)
        .bind(member_type)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query("UPDATE users SET current_school_id = $1, updated_at = now() WHERE id = $2")
            .bind(&school_id)
            .bind(&user_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        self.find_by_id(&IdType::from_string(user_id))
            .await?
            .ok_or(AppError {
                message: "User not found after school update".into(),
            })
    }

    pub async fn remove_school_from_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_id = Self::id_to_string(user_id)?;
        let school_id = Self::id_to_string(school_id)?;

        sqlx::query(
            r#"
            UPDATE school_memberships
            SET status = 'removed', ended_at = now(), updated_at = now()
            WHERE user_id = $1 AND school_id = $2
            "#,
        )
        .bind(&user_id)
        .bind(&school_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        sqlx::query(
            r#"
            UPDATE users
            SET current_school_id = NULL, updated_at = now()
            WHERE id = $1 AND current_school_id = $2
            "#,
        )
        .bind(&user_id)
        .bind(&school_id)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_by_id(&IdType::from_string(user_id))
            .await?
            .ok_or(AppError {
                message: "User not found after school removal".into(),
            })
    }
}
