use actix_web::web;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::{Address, Contact, Paginated, RelatedUser, SocialMedia, UserRole},
        school::{
            AffiliationType, AttendanceSystemType, School, SchoolAcademicRequest,
            SchoolAcademicResponse, SchoolMemberType, SchoolPartial, SchoolType,
        },
    },
    errors::AppError,
    mappers::school_mapper::to_school_school_token,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    repositories::user_repo::UserRepo,
    services::cloudinary_service::CloudinaryService,
    utils::{
        code::generate_code,
        names::{is_valid_name, is_valid_username},
        object_id::{parse_object_id_value, ObjectId},
        school_token::{create_school_token, verify_school_token},
    },
};

#[derive(Debug, Clone)]
pub enum SchoolLookup {
    Id(String),
    Username(String),
    Code(String),
    Name(String),
    CreatorId(String),
}

pub struct SchoolService {
    pub pool: PgPool,
}

impl SchoolService {
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

    fn default_database_name(id: &str, database_name: Option<String>) -> String {
        database_name.unwrap_or_else(|| format!("school_{}", id))
    }

    fn lookup_from_query(query: &RequestQuery) -> Result<Option<SchoolLookup>, AppError> {
        if let Some(id) = query.by_ids.first() {
            parse_object_id_value(id)?;
            return Ok(Some(SchoolLookup::Id(id.clone())));
        }

        if query.field.is_empty() {
            return Ok(None);
        }
        if query.field.len() != query.value.len() {
            return Err(AppError {
                message: "Number of fields must match number of values".into(),
            });
        }

        let field = query.field[0].as_str();
        let value = query.value[0].clone();
        match field {
            "_id" | "id" => {
                parse_object_id_value(&value)?;
                Ok(Some(SchoolLookup::Id(value)))
            }
            "username" => Ok(Some(SchoolLookup::Username(value))),
            "code" => Ok(Some(SchoolLookup::Code(value))),
            "name" => Ok(Some(SchoolLookup::Name(value))),
            "creator_id" => {
                parse_object_id_value(&value)?;
                Ok(Some(SchoolLookup::CreatorId(value)))
            }
            _ => Err(AppError {
                message: format!("Unsupported school filter field: {}", field),
            }),
        }
    }

    fn push_lookup<'a>(query: &mut QueryBuilder<'a, Postgres>, lookup: &'a SchoolLookup) {
        match lookup {
            SchoolLookup::Id(id) => {
                query.push(" AND id = ").push_bind(id);
            }
            SchoolLookup::Username(username) => {
                query
                    .push(" AND lower(username) = lower(")
                    .push_bind(username)
                    .push(")");
            }
            SchoolLookup::Code(code) => {
                query
                    .push(" AND lower(code) = lower(")
                    .push_bind(code)
                    .push(")");
            }
            SchoolLookup::Name(name) => {
                query
                    .push(" AND lower(name) = lower(")
                    .push_bind(name)
                    .push(")");
            }
            SchoolLookup::CreatorId(creator_id) => {
                query.push(" AND creator_id = ").push_bind(creator_id);
            }
        }
    }

    fn push_search<'a>(query: &mut QueryBuilder<'a, Postgres>, filter: Option<&'a str>) {
        let Some(filter) = filter else {
            return;
        };
        if filter.trim().is_empty() {
            return;
        }

        let search = format!("%{}%", filter.to_lowercase());
        query
            .push(" AND (lower(name) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(username) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(coalesce(description, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(coalesce(code, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(id) LIKE ")
            .push_bind(search)
            .push(")");
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

    fn contact_from_row(row: &PgRow) -> Option<Contact> {
        let email: Option<String> = row.try_get("contact_email").ok().flatten();
        let phone: Option<String> = row.try_get("contact_phone").ok().flatten();
        let alt_phone: Option<String> = row.try_get("contact_alt_phone").ok().flatten();
        let whatsapp: Option<String> = row.try_get("contact_whatsapp").ok().flatten();

        (email.is_some() || phone.is_some() || alt_phone.is_some() || whatsapp.is_some()).then_some(
            Contact {
                email,
                phone,
                alt_phone,
                whatsapp,
            },
        )
    }

    async fn fetch_id_relation(
        &self,
        table: &str,
        id_column: &str,
        school_id: &str,
    ) -> Result<Option<Vec<ObjectId>>, AppError> {
        let sql = format!(
            "SELECT {} FROM {} WHERE school_id = $1 ORDER BY position ASC, created_at ASC",
            id_column, table
        );
        let rows = sqlx::query(&sql)
            .bind(school_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let ids = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get(id_column).map_err(Self::db_error)?;
                Self::parse_oid(&id, id_column)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((!ids.is_empty()).then_some(ids))
    }

    async fn fetch_social_media(
        &self,
        school_id: &str,
    ) -> Result<Option<Vec<SocialMedia>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT platform, url
            FROM school_social_media
            WHERE school_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(school_id)
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
        school_id: &str,
        value_type: &str,
    ) -> Result<Option<Vec<String>>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT value
            FROM school_profile_values
            WHERE school_id = $1 AND value_type = $2
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(school_id)
        .bind(value_type)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let values = rows
            .into_iter()
            .map(|row| row.try_get("value").map_err(Self::db_error))
            .collect::<Result<Vec<String>, AppError>>()?;

        Ok((!values.is_empty()).then_some(values))
    }

    async fn school_from_row(&self, row: PgRow) -> Result<School, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;

        Ok(School {
            id: Some(Self::parse_oid(&id, "id")?),
            creator_id: Self::parse_oid_opt(
                row.try_get("creator_id").ok().flatten(),
                "creator_id",
            )?,
            username: row.try_get("username").map_err(Self::db_error)?,
            logo: row.try_get("logo").ok().flatten(),
            logo_id: row.try_get("logo_id").ok().flatten(),
            name: row.try_get("name").map_err(Self::db_error)?,
            code: row.try_get("code").ok().flatten(),
            description: row.try_get("description").ok().flatten(),
            school_type: Self::enum_from_string::<SchoolType>(
                row.try_get("school_type").ok().flatten(),
            ),
            curriculum: self
                .fetch_id_relation("school_curricula", "curriculum_id", &id)
                .await?,
            education_level: self
                .fetch_id_relation("school_education_levels", "education_level_id", &id)
                .await?,
            accreditation_number: row.try_get("accreditation_number").ok().flatten(),
            affiliation: Self::enum_from_string::<AffiliationType>(
                row.try_get("affiliation").ok().flatten(),
            ),
            school_members: Self::enum_from_string::<SchoolMemberType>(
                row.try_get("school_members").ok().flatten(),
            ),
            address: Self::address_from_row(&row)?,
            contact: Self::contact_from_row(&row),
            website: row.try_get("website").ok().flatten(),
            social_media: self.fetch_social_media(&id).await?,
            student_capacity: row.try_get("student_capacity").ok().flatten(),
            uniform_required: row.try_get("uniform_required").ok().flatten(),
            attendance_system: Self::enum_from_string::<AttendanceSystemType>(
                row.try_get("attendance_system").ok().flatten(),
            ),
            scholarship_available: row.try_get("scholarship_available").ok().flatten(),
            classrooms: row.try_get("classrooms").ok().flatten(),
            library: row.try_get("library").ok().flatten(),
            labs: self.fetch_profile_values(&id, "labs").await?,
            sports_extracurricular: self
                .fetch_profile_values(&id, "sports_extracurricular")
                .await?,
            online_classes: row.try_get("online_classes").ok().flatten(),
            database_name: Some(Self::default_database_name(
                &id,
                row.try_get("database_name").ok().flatten(),
            )),
            is_active: row.try_get("is_active").ok().flatten(),
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
        })
    }

    async fn find_one_by_lookup(&self, lookup: SchoolLookup) -> Result<Option<School>, AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT * FROM schools WHERE deleted_at IS NULL");
        Self::push_lookup(&mut query, &lookup);

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.school_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(&self, dto: School) -> Result<School, AppError> {
        self.ensure_indexes().await?;
        is_valid_username(&dto.username).map_err(|e| AppError { message: e })?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;

        if let Some(existing) = self
            .find_one_by_lookup(SchoolLookup::Username(dto.username.clone()))
            .await?
        {
            return Err(AppError {
                message: format!("School username already exists: {}", existing.username),
            });
        }

        let mut new_school = dto.clone();
        let id = new_school
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let database_name = Some(Self::default_database_name(
            &id,
            new_school.database_name.clone(),
        ));
        new_school.is_active = Some(false);
        new_school.code = Some(new_school.code.clone().unwrap_or_else(generate_code));

        if let Some(logo_file) = new_school.logo.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&logo_file)
                .await
                .map_err(|e| AppError { message: e })?;
            new_school.logo_id = Some(cloud_res.public_id);
            new_school.logo = Some(cloud_res.secure_url);
        }

        let address = new_school.address.as_ref();
        let contact = new_school.contact.as_ref();

        sqlx::query(
            r#"
            INSERT INTO schools (
              id, creator_id, username, name, code, logo, logo_id, description, school_type,
              affiliation, database_name, is_active, accreditation_number, school_members, website,
              student_capacity, uniform_required, attendance_system, scholarship_available,
              classrooms, library, online_classes, contact_email, contact_phone, contact_alt_phone,
              contact_whatsapp, address_country, address_province, address_district, address_sector,
              address_cell, address_village, address_state, address_street, address_city,
              address_postal_code, address_google_map_url, created_at, updated_at
            )
            VALUES (
              $1, $2, $3, $4, $5, $6, $7, $8, $9,
              $10, $11, $12, $13, $14, $15,
              $16, $17, $18, $19,
              $20, $21, $22, $23, $24, $25,
              $26, $27, $28, $29, $30,
              $31, $32, $33, $34, $35,
              $36, $37, now(), now()
            )
            "#,
        )
        .bind(&id)
        .bind(new_school.creator_id.as_ref().map(|id| id.to_hex()))
        .bind(&new_school.username)
        .bind(&new_school.name)
        .bind(&new_school.code)
        .bind(&new_school.logo)
        .bind(&new_school.logo_id)
        .bind(&new_school.description)
        .bind(
            new_school
                .school_type
                .as_ref()
                .and_then(Self::enum_to_string),
        )
        .bind(
            new_school
                .affiliation
                .as_ref()
                .and_then(Self::enum_to_string),
        )
        .bind(&database_name)
        .bind(new_school.is_active.unwrap_or(false))
        .bind(&new_school.accreditation_number)
        .bind(
            new_school
                .school_members
                .as_ref()
                .and_then(Self::enum_to_string),
        )
        .bind(&new_school.website)
        .bind(new_school.student_capacity)
        .bind(new_school.uniform_required)
        .bind(
            new_school
                .attendance_system
                .as_ref()
                .and_then(Self::enum_to_string),
        )
        .bind(new_school.scholarship_available)
        .bind(new_school.classrooms)
        .bind(new_school.library)
        .bind(new_school.online_classes)
        .bind(contact.and_then(|contact| contact.email.clone()))
        .bind(contact.and_then(|contact| contact.phone.clone()))
        .bind(contact.and_then(|contact| contact.alt_phone.clone()))
        .bind(contact.and_then(|contact| contact.whatsapp.clone()))
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
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_school_relations(&id, &new_school).await?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        lookup: Option<SchoolLookup>,
    ) -> Result<School, AppError> {
        let lookup = match id {
            Some(id) => SchoolLookup::Id(Self::id_to_string(id)?),
            None => lookup.ok_or_else(|| AppError {
                message: "School lookup is required".into(),
            })?,
        };

        self.find_one_by_lookup(lookup).await?.ok_or(AppError {
            message: "School not found".into(),
        })
    }

    pub async fn find_by_code(&self, code: &str) -> Result<School, AppError> {
        self.find_one(None, Some(SchoolLookup::Code(code.to_string())))
            .await
    }

    pub async fn find_by_request_query(&self, query: &RequestQuery) -> Result<School, AppError> {
        self.find_one(None, Self::lookup_from_query(query)?).await
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        lookup: Option<SchoolLookup>,
    ) -> Result<Paginated<School>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM schools WHERE deleted_at IS NULL");
        Self::push_search(&mut count_query, filter.as_deref());
        if let Some(lookup) = &lookup {
            Self::push_lookup(&mut count_query, lookup);
        }

        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut query =
            QueryBuilder::<Postgres>::new("SELECT * FROM schools WHERE deleted_at IS NULL");
        Self::push_search(&mut query, filter.as_deref());
        if let Some(lookup) = &lookup {
            Self::push_lookup(&mut query, lookup);
        }
        query
            .push(" ORDER BY created_at DESC LIMIT ")
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
            data.push(self.school_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn get_all_for_query(
        &self,
        query: &RequestQuery,
    ) -> Result<Paginated<School>, AppError> {
        self.get_all(
            query.filter.clone(),
            query.limit,
            query.skip,
            Self::lookup_from_query(query)?,
        )
        .await
    }

    pub async fn update(&self, id: &IdType, update: &SchoolPartial) -> Result<School, AppError> {
        if let Some(ref name) = update.name {
            is_valid_name(name).map_err(|e| AppError { message: e })?;
        }
        if let Some(ref username) = update.username {
            is_valid_username(username).map_err(|e| AppError { message: e })?;
        }

        let existing = self.find_one(Some(id), None).await?;

        if let Some(ref username) = update.username {
            if existing.username != *username
                && self
                    .find_one_by_lookup(SchoolLookup::Username(username.clone()))
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("School username already exists: {}", username),
                });
            }
        }

        let mut update_data = update.clone();
        if let Some(Some(new_logo_data)) = update.logo.clone() {
            if Some(new_logo_data.clone()) != existing.logo {
                if let Some(old_logo_id) = existing.logo_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_logo_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_logo_data)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.logo_id = Some(Some(cloud_res.public_id));
                update_data.logo = Some(Some(cloud_res.secure_url));
            }
        }

        let id = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE schools SET updated_at = now()");
        let mut touched_scalar = false;

        macro_rules! set_required {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched_scalar = true;
                }
            };
        }
        macro_rules! set_optional {
            ($field:ident, $column:literal) => {
                if let Some(value) = update_data.$field.clone() {
                    sql.push(", ").push($column).push(" = ").push_bind(value);
                    touched_scalar = true;
                }
            };
        }

        set_required!(username, "username");
        set_required!(name, "name");
        set_optional!(logo, "logo");
        set_optional!(logo_id, "logo_id");
        set_optional!(code, "code");
        set_optional!(description, "description");
        set_optional!(accreditation_number, "accreditation_number");
        set_optional!(website, "website");
        set_optional!(database_name, "database_name");
        set_optional!(is_active, "is_active");
        set_optional!(student_capacity, "student_capacity");
        set_optional!(uniform_required, "uniform_required");
        set_optional!(scholarship_available, "scholarship_available");
        set_optional!(classrooms, "classrooms");
        set_optional!(library, "library");
        set_optional!(online_classes, "online_classes");

        if let Some(value) = update_data.creator_id.clone() {
            sql.push(", creator_id = ")
                .push_bind(value.map(|id| id.to_hex()));
            touched_scalar = true;
        }
        if let Some(value) = update_data.school_type.clone() {
            sql.push(", school_type = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched_scalar = true;
        }
        if let Some(value) = update_data.affiliation.clone() {
            sql.push(", affiliation = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched_scalar = true;
        }
        if let Some(value) = update_data.school_members.clone() {
            sql.push(", school_members = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched_scalar = true;
        }
        if let Some(value) = update_data.attendance_system.clone() {
            sql.push(", attendance_system = ")
                .push_bind(value.as_ref().and_then(Self::enum_to_string));
            touched_scalar = true;
        }
        if let Some(address) = update_data.address.clone() {
            sql.push(", address_country = ")
                .push_bind(address.as_ref().map(|a| a.country.clone()))
                .push(", address_province = ")
                .push_bind(address.as_ref().and_then(|a| a.province.clone()))
                .push(", address_district = ")
                .push_bind(address.as_ref().and_then(|a| a.district.clone()))
                .push(", address_sector = ")
                .push_bind(address.as_ref().and_then(|a| a.sector.clone()))
                .push(", address_cell = ")
                .push_bind(address.as_ref().and_then(|a| a.cell.clone()))
                .push(", address_village = ")
                .push_bind(address.as_ref().and_then(|a| a.village.clone()))
                .push(", address_state = ")
                .push_bind(address.as_ref().and_then(|a| a.state.clone()))
                .push(", address_street = ")
                .push_bind(address.as_ref().and_then(|a| a.street.clone()))
                .push(", address_city = ")
                .push_bind(address.as_ref().and_then(|a| a.city.clone()))
                .push(", address_postal_code = ")
                .push_bind(address.as_ref().and_then(|a| a.postal_code.clone()))
                .push(", address_google_map_url = ")
                .push_bind(address.as_ref().and_then(|a| a.google_map_url.clone()));
            touched_scalar = true;
        }
        if let Some(contact) = update_data.contact.clone() {
            sql.push(", contact_email = ")
                .push_bind(contact.as_ref().and_then(|c| c.email.clone()))
                .push(", contact_phone = ")
                .push_bind(contact.as_ref().and_then(|c| c.phone.clone()))
                .push(", contact_alt_phone = ")
                .push_bind(contact.as_ref().and_then(|c| c.alt_phone.clone()))
                .push(", contact_whatsapp = ")
                .push_bind(contact.as_ref().and_then(|c| c.whatsapp.clone()));
            touched_scalar = true;
        }

        sql.push(" WHERE id = ")
            .push_bind(&id)
            .push(" AND deleted_at IS NULL");
        let result = sql
            .build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let touched_relations = update_data.curriculum.is_some()
            || update_data.education_level.is_some()
            || update_data.social_media.is_some()
            || update_data.labs.is_some()
            || update_data.sports_extracurricular.is_some();

        if result.rows_affected() == 0 {
            return Err(AppError {
                message: "School not found".into(),
            });
        }
        if !touched_scalar && !touched_relations {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.sync_school_partial_relations(&id, &update_data)
            .await?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<School, AppError> {
        let school = self.find_one(Some(id), None).await?;
        let id = Self::id_to_string(id)?;
        sqlx::query("DELETE FROM schools WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(school)
    }

    pub async fn count_for_query(&self, query: &RequestQuery) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all_for_query(query).await?.total as u64,
        })
    }

    pub async fn refresh_school_token(&self, token: &str) -> Result<String, AppError> {
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_school_token(&token_clean).ok_or_else(|| AppError {
            message: "Invalid token".to_string(),
        })?;

        let school_id = IdType::from_string(&claims.id);
        let school = self.find_one(Some(&school_id), None).await?;
        let dto = to_school_school_token(&school, claims.member)?;

        Ok(create_school_token(dto))
    }

    pub async fn create_school_token(
        &self,
        id: &IdType,
        member: &AuthUserDto,
        state: &AppState,
    ) -> Result<String, AppError> {
        let school = self.find_one(Some(id), None).await?;
        let member_id = parse_object_id_value(&member.id)?;
        let user_repo = UserRepo::new(&state.pg.pool);

        let school_member = match member.role.as_ref() {
            Some(UserRole::SCHOOLSTAFF) if school.creator_id == Some(member_id) => user_repo
                .find_by_id(&IdType::from_object_id(member_id))
                .await?
                .map(RelatedUser::USER),
            _ => None,
        };

        let school_token = to_school_school_token(&school, school_member)?;

        Ok(create_school_token(school_token))
    }

    pub async fn create_school_token_from_member(
        &self,
        id: &IdType,
        school_member: Option<RelatedUser>,
    ) -> Result<String, AppError> {
        let school = self.find_one(Some(id), None).await?;
        let school_token = to_school_school_token(&school, school_member)?;

        Ok(create_school_token(school_token))
    }

    pub async fn setup_school_academics(
        &self,
        school_id: &IdType,
        _req: SchoolAcademicRequest,
        _state: web::Data<AppState>,
        _logged_user: AuthUserDto,
    ) -> Result<SchoolAcademicResponse, AppError> {
        self.find_one(Some(school_id), None).await?;
        Err(AppError {
            message:
                "School academic setup is pending the classes and subjects PostgreSQL migration"
                    .to_string(),
        })
    }

    pub async fn search_members(
        &self,
        school_id: &str,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<RelatedUser>, AppError> {
        parse_object_id_value(school_id)?;
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search = filter.unwrap_or_default();
        let search_like = format!("%{}%", search.to_lowercase());

        let mut count_query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT count(*)
            FROM school_memberships sm
            JOIN users u ON u.id = sm.user_id
            WHERE sm.school_id =
            "#,
        );
        count_query
            .push_bind(school_id)
            .push(" AND sm.status = 'active' AND u.deleted_at IS NULL");
        if !search.is_empty() {
            count_query
                .push(" AND (lower(u.name) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(u.email) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(coalesce(u.username, '')) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(coalesce(u.role, '')) LIKE ")
                .push_bind(search_like.clone())
                .push(")");
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT u.*
            FROM school_memberships sm
            JOIN users u ON u.id = sm.user_id
            WHERE sm.school_id =
            "#,
        );
        query
            .push_bind(school_id)
            .push(" AND sm.status = 'active' AND u.deleted_at IS NULL");
        if !search.is_empty() {
            query
                .push(" AND (lower(u.name) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(u.email) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(coalesce(u.username, '')) LIKE ")
                .push_bind(search_like.clone())
                .push(" OR lower(coalesce(u.role, '')) LIKE ")
                .push_bind(search_like)
                .push(")");
        }
        query
            .push(" ORDER BY u.name ASC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let user_repo = UserRepo::new(&self.pool);
        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(RelatedUser::USER(user_repo.user_from_sql_row(row).await?));
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    async fn sync_id_relation(
        &self,
        table: &str,
        id_column: &str,
        school_id: &str,
        ids: &[ObjectId],
    ) -> Result<(), AppError> {
        let delete_sql = format!("DELETE FROM {} WHERE school_id = $1", table);
        sqlx::query(&delete_sql)
            .bind(school_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let insert_sql = format!(
            "INSERT INTO {} (id, school_id, {}, position) VALUES ($1, $2, $3, $4)",
            table, id_column
        );
        for (position, id) in ids.iter().enumerate() {
            sqlx::query(&insert_sql)
                .bind(Self::new_id())
                .bind(school_id)
                .bind(id.to_hex())
                .bind(position as i32)
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn sync_social_media(
        &self,
        school_id: &str,
        social_media: &[SocialMedia],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM school_social_media WHERE school_id = $1")
            .bind(school_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, item) in social_media.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO school_social_media (id, school_id, platform, url, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(school_id)
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
        school_id: &str,
        value_type: &str,
        values: &[String],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM school_profile_values WHERE school_id = $1 AND value_type = $2")
            .bind(school_id)
            .bind(value_type)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for (position, value) in values.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO school_profile_values (id, school_id, value_type, value, position)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(Self::new_id())
            .bind(school_id)
            .bind(value_type)
            .bind(value)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn sync_school_relations(
        &self,
        school_id: &str,
        school: &School,
    ) -> Result<(), AppError> {
        if let Some(ids) = &school.curriculum {
            self.sync_id_relation("school_curricula", "curriculum_id", school_id, ids)
                .await?;
        }
        if let Some(ids) = &school.education_level {
            self.sync_id_relation(
                "school_education_levels",
                "education_level_id",
                school_id,
                ids,
            )
            .await?;
        }
        if let Some(social_media) = &school.social_media {
            self.sync_social_media(school_id, social_media).await?;
        }
        if let Some(labs) = &school.labs {
            self.replace_profile_values(school_id, "labs", labs).await?;
        }
        if let Some(sports) = &school.sports_extracurricular {
            self.replace_profile_values(school_id, "sports_extracurricular", sports)
                .await?;
        }

        Ok(())
    }

    async fn sync_school_partial_relations(
        &self,
        school_id: &str,
        school: &SchoolPartial,
    ) -> Result<(), AppError> {
        if let Some(Some(ids)) = &school.curriculum {
            self.sync_id_relation("school_curricula", "curriculum_id", school_id, ids)
                .await?;
        }
        if let Some(Some(ids)) = &school.education_level {
            self.sync_id_relation(
                "school_education_levels",
                "education_level_id",
                school_id,
                ids,
            )
            .await?;
        }
        if let Some(Some(social_media)) = &school.social_media {
            self.sync_social_media(school_id, social_media).await?;
        }
        if let Some(Some(labs)) = &school.labs {
            self.replace_profile_values(school_id, "labs", labs).await?;
        }
        if let Some(Some(sports)) = &school.sports_extracurricular {
            self.replace_profile_values(school_id, "sports_extracurricular", sports)
                .await?;
        }

        Ok(())
    }
}
