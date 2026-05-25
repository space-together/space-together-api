use chrono::Utc;
use serde::de::DeserializeOwned;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        role::{
            Permission, PermissionScope, Role, RolePartial, RoleType, RoleWithRelations,
            UserRoleAssignment,
        },
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    services::school_service::SchoolService,
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct RoleQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl RoleQuery {
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

pub struct RoleService {
    pub pool: PgPool,
}

impl RoleService {
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

    fn enum_from_string<T: DeserializeOwned>(value: Option<String>) -> Option<T> {
        value.and_then(|raw| serde_json::from_value(serde_json::Value::String(raw)).ok())
    }

    fn role_type_to_string(value: &RoleType) -> String {
        match value {
            RoleType::System => "System",
            RoleType::Custom => "Custom",
        }
        .to_string()
    }

    fn role_code(name: &str) -> String {
        name.trim()
            .to_ascii_lowercase()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, name, code, description, role_type, is_active, created_at, updated_at
        FROM roles
        WHERE 1 = 1
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RoleQuery>,
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
                "role_type" => {
                    sql.push(" AND lower(role_type) = lower(")
                        .push_bind(value)
                        .push(")");
                }
                "is_active" => {
                    let parsed = value.parse::<bool>().map_err(|_| AppError {
                        message: "Invalid is_active filter value".into(),
                    })?;
                    sql.push(" AND is_active = ").push_bind(parsed);
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported role filter field: {}", field),
                    });
                }
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(name) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(code) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(description, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(role_type) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(id) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    async fn fetch_permissions(&self, role_id: &str) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT p.code
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.code ASC
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter()
            .map(|row| row.try_get("code").map_err(Self::db_error))
            .collect()
    }

    async fn sync_permissions(
        &self,
        role_id: &str,
        permissions: &[String],
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        for permission in permissions {
            let permission_id = self.ensure_permission(permission, None).await?;
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(role_id)
            .bind(permission_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        }

        Ok(())
    }

    async fn ensure_permission(
        &self,
        code: &str,
        description: Option<&str>,
    ) -> Result<String, AppError> {
        let existing =
            sqlx::query_scalar::<_, String>("SELECT id FROM permissions WHERE code = $1")
                .bind(code)
                .fetch_optional(&self.pool)
                .await
                .map_err(Self::db_error)?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let id = Self::new_id();
        sqlx::query("INSERT INTO permissions (id, code, description) VALUES ($1, $2, $3)")
            .bind(&id)
            .bind(code)
            .bind(description)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(id)
    }

    async fn role_from_row(&self, row: PgRow) -> Result<Role, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Role {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            name: row.try_get("name").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            role_type: Self::enum_from_string::<RoleType>(row.try_get("role_type").ok().flatten())
                .unwrap_or(RoleType::Custom),
            permissions: self.fetch_permissions(&id).await?,
            is_active: row.try_get("is_active").ok().flatten().unwrap_or(true),
            created_at: row.try_get("created_at").ok().unwrap_or_else(Utc::now),
            updated_at: row.try_get("updated_at").ok().unwrap_or_else(Utc::now),
        })
    }

    async fn find_duplicate(
        &self,
        name: &str,
        school_id: Option<&str>,
        except_id: Option<&str>,
    ) -> Result<Option<Role>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(Self::select_sql());
        query
            .push(" AND lower(name) = lower(")
            .push_bind(name)
            .push(")");
        match school_id {
            Some(school_id) => {
                query.push(" AND school_id = ").push_bind(school_id);
            }
            None => {
                query.push(" AND school_id IS NULL");
            }
        }
        if let Some(except_id) = except_id {
            query.push(" AND id <> ").push_bind(except_id);
        }
        query.push(" LIMIT 1");

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;

        match row {
            Some(row) => Ok(Some(self.role_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub async fn seed_default_permissions(&self) -> Result<(), AppError> {
        for permission in Self::get_default_permissions() {
            self.ensure_permission(&permission.name, permission.description.as_deref())
                .await?;
        }
        Ok(())
    }

    pub async fn create_role(&self, mut dto: Role) -> Result<Role, AppError> {
        self.ensure_indexes().await?;
        self.seed_default_permissions().await?;

        if dto.name.trim().is_empty() {
            return Err(AppError {
                message: "Role name cannot be empty".into(),
            });
        }

        let school_id = dto.school_id.as_ref().map(|id| id.to_hex());
        if self
            .find_duplicate(&dto.name, school_id.as_deref(), None)
            .await?
            .is_some()
        {
            return Err(AppError {
                message: format!("Role with name '{}' already exists", dto.name),
            });
        }

        let id = dto
            .id
            .take()
            .map(|id| id.to_hex())
            .unwrap_or_else(Self::new_id);
        let code = Self::role_code(&dto.name);

        sqlx::query(
            r#"
            INSERT INTO roles (
              id, school_id, name, code, description, role_type, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(&dto.name)
        .bind(code)
        .bind(&dto.description)
        .bind(Self::role_type_to_string(&dto.role_type))
        .bind(dto.is_active)
        .bind(dto.created_at)
        .bind(dto.updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.sync_permissions(&id, &dto.permissions).await?;
        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<RoleQuery>,
    ) -> Result<Role, AppError> {
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
            Some(row) => Ok(self.role_from_row(row).await?),
            None => Err(AppError {
                message: "Role not found".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<RoleQuery>,
    ) -> Result<Paginated<Role>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM roles WHERE 1 = 1");
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
            data.push(self.role_from_row(row).await?);
        }

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update_role(&self, id: &IdType, update: &RolePartial) -> Result<Role, AppError> {
        let existing_role = self.find_one(Some(id), None).await?;
        if existing_role.role_type == RoleType::System {
            return Err(AppError {
                message: "Cannot modify system roles".into(),
            });
        }

        if let Some(ref name) = update.name {
            if name.trim().is_empty() {
                return Err(AppError {
                    message: "Role name cannot be empty".into(),
                });
            }
            if existing_role.name != *name
                && self
                    .find_duplicate(
                        name,
                        existing_role
                            .school_id
                            .as_ref()
                            .map(|id| id.to_hex())
                            .as_deref(),
                        Some(&Self::id_to_string(id)?),
                    )
                    .await?
                    .is_some()
            {
                return Err(AppError {
                    message: format!("Role with name '{}' already exists", name),
                });
            }
        }

        let id_string = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE roles SET updated_at = now()");
        let mut touched = false;

        if let Some(name) = &update.name {
            sql.push(", name = ")
                .push_bind(name)
                .push(", code = ")
                .push_bind(Self::role_code(name));
            touched = true;
        }
        if let Some(description) = &update.description {
            sql.push(", description = ").push_bind(description);
            touched = true;
        }
        if let Some(role_type) = update.role_type {
            sql.push(", role_type = ")
                .push_bind(Self::role_type_to_string(&role_type));
            touched = true;
        }
        if let Some(is_active) = update.is_active {
            sql.push(", is_active = ").push_bind(is_active);
            touched = true;
        }
        if let Some(school_id) = update.school_id {
            sql.push(", school_id = ")
                .push_bind(school_id.map(|id| id.to_hex()));
            touched = true;
        }

        if touched {
            sql.push(" WHERE id = ").push_bind(&id_string);
            sql.build()
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        if let Some(permissions) = &update.permissions {
            self.sync_permissions(&id_string, permissions).await?;
        }

        if !touched && update.permissions.is_none() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        self.find_one(Some(id), None).await
    }

    pub async fn delete_role(&self, id: &IdType) -> Result<Role, AppError> {
        let role = self.find_one(Some(id), None).await?;
        if role.role_type == RoleType::System {
            return Err(AppError {
                message: "Cannot delete system roles".into(),
            });
        }

        let id_string = Self::id_to_string(id)?;
        let assignment_count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM user_role_assignments WHERE role_id = $1")
                .bind(&id_string)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::db_error)?;

        if assignment_count > 0 {
            return Err(AppError {
                message: format!(
                    "Cannot delete role. It is assigned to {} user(s)",
                    assignment_count
                ),
            });
        }

        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id_string)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(role)
    }

    pub async fn assign_role_to_user(
        &self,
        user_id: &IdType,
        role_id: &IdType,
        school_id: &IdType,
    ) -> Result<UserRoleAssignment, AppError> {
        let role = self.find_one(Some(role_id), None).await?;
        if !role.is_active {
            return Err(AppError {
                message: "Cannot assign inactive role".into(),
            });
        }

        let user_id_string = Self::id_to_string(user_id)?;
        let role_id_string = Self::id_to_string(role_id)?;
        let school_id_string = Self::id_to_string(school_id)?;

        if let Some(role_school_id) = role.school_id {
            if role_school_id.to_hex() != school_id_string {
                return Err(AppError {
                    message: "Cannot assign role from different school".into(),
                });
            }
        }

        let exists: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)
            FROM user_role_assignments
            WHERE user_id = $1 AND role_id = $2 AND school_id = $3 AND revoked_at IS NULL
            "#,
        )
        .bind(&user_id_string)
        .bind(&role_id_string)
        .bind(&school_id_string)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if exists > 0 {
            return Err(AppError {
                message: "User already has this role assigned".into(),
            });
        }

        let id = Self::new_id();
        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (
              id, user_id, school_id, role_id, role, scope, starts_at, assigned_at
            )
            VALUES ($1, $2, $3, $4, $5, 'school', now(), now())
            "#,
        )
        .bind(&id)
        .bind(&user_id_string)
        .bind(&school_id_string)
        .bind(&role_id_string)
        .bind(&role.name)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(UserRoleAssignment {
            id: Some(Self::parse_oid(&id, "id")?),
            user_id: Self::parse_oid(&user_id_string, "user_id")?,
            role_id: Self::parse_oid(&role_id_string, "role_id")?,
            school_id: Self::parse_oid(&school_id_string, "school_id")?,
            assigned_at: Utc::now(),
        })
    }

    pub async fn user_has_permission(
        &self,
        user_id: &IdType,
        school_id: &IdType,
        permission: &str,
    ) -> Result<bool, AppError> {
        let user_id = Self::id_to_string(user_id)?;
        let school_id = Self::id_to_string(school_id)?;

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)
            FROM user_role_assignments ura
            JOIN role_permissions rp ON rp.role_id = ura.role_id
            JOIN permissions p ON p.id = rp.permission_id
            JOIN roles r ON r.id = ura.role_id
            WHERE ura.user_id = $1
              AND ura.school_id = $2
              AND ura.revoked_at IS NULL
              AND r.is_active = true
              AND p.code = $3
            "#,
        )
        .bind(user_id)
        .bind(school_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(count > 0)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<RoleQuery>,
    ) -> Result<Paginated<RoleWithRelations>, AppError> {
        let roles = self.get_all(filter, limit, skip, query).await?;
        let mut data = Vec::with_capacity(roles.data.len());
        for role in roles.data {
            data.push(self.with_relations(role).await?);
        }

        Ok(Paginated {
            data,
            total: roles.total,
            total_pages: roles.total_pages,
            current_page: roles.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<RoleQuery>,
    ) -> Result<RoleWithRelations, AppError> {
        let role = self.find_one(id, query).await?;
        self.with_relations(role).await
    }

    async fn with_relations(&self, role: Role) -> Result<RoleWithRelations, AppError> {
        let school = match role.school_id {
            Some(school_id) => SchoolService::new(&self.pool)
                .find_one(Some(&IdType::from_object_id(school_id)), None)
                .await
                .ok(),
            None => None,
        };

        Ok(RoleWithRelations { role, school })
    }

    pub async fn count_roles(
        &self,
        filter: Option<String>,
        query: Option<RoleQuery>,
    ) -> Result<CountDoc, AppError> {
        Ok(CountDoc {
            count: self.get_all(filter, Some(1), Some(0), query).await?.total as u64,
        })
    }

    pub fn get_default_permissions() -> Vec<Permission> {
        vec![
            Permission {
                name: "assignment.create".to_string(),
                description: Some("Create assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.read.own".to_string(),
                description: Some("Read own assignments".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "assignment.read.class".to_string(),
                description: Some("Read class assignments".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "assignment.read.school".to_string(),
                description: Some("Read all school assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.update".to_string(),
                description: Some("Update assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.delete".to_string(),
                description: Some("Delete assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "submission.grade".to_string(),
                description: Some("Grade submissions".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "submission.read.own".to_string(),
                description: Some("Read own submissions".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "submission.read.class".to_string(),
                description: Some("Read class submissions".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "submission.read.school".to_string(),
                description: Some("Read all school submissions".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "parent.read.child.assignment".to_string(),
                description: Some("Read child's assignments".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "parent.read.child.submission".to_string(),
                description: Some("Read child's submissions".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "role.assign".to_string(),
                description: Some("Assign roles to users".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.create".to_string(),
                description: Some("Create new roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.update".to_string(),
                description: Some("Update roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.delete".to_string(),
                description: Some("Delete roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "feature.toggle".to_string(),
                description: Some("Toggle features".to_string()),
                scope: PermissionScope::School,
            },
        ]
    }
}
