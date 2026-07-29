use crate::config::state::AppState;
use crate::domain::auth_user::AuthUserDto;
use crate::models::school_token_model::SchoolToken;
use actix_web::{web, HttpMessage, HttpRequest};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_id: Option<String>,
    pub role: Option<String>,
    pub school_id: Option<String>,
    pub school_database_name: Option<String>,
}

impl RequestContext {
    pub fn is_school_scoped(&self) -> bool {
        self.school_id.is_some()
    }
}

pub fn postgres_pool(state: &web::Data<AppState>) -> &PgPool {
    &state.pg.pool
}

pub fn request_context(req: &HttpRequest) -> RequestContext {
    let extensions = req.extensions();
    let school_token = extensions.get::<SchoolToken>();
    let auth_user = extensions.get::<AuthUserDto>();

    RequestContext {
        user_id: auth_user.map(|user| user.id.clone()),
        role: auth_user
            .and_then(|user| user.role.as_ref())
            .map(|role| format!("{role:?}")),
        school_id: school_token.map(|token| token.id.clone()).or_else(|| {
            req.headers()
                .get("x-school-id")
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned)
        }),
        school_database_name: school_token.map(|token| token.database_name.clone()),
    }
}
