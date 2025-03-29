use mongodb::{bson::doc, Database, IndexModel};

use crate::{
    libs::auth::user_session::SESSION_EXPIRATION_SECONDS,
    models::auth::session_model::UserSessionModel,
};

pub async fn collection_expires(db: &Database) -> Result<(), String> {
    let index = IndexModel::builder()
        .keys(doc! { "expires_at": 1 }) // Index on 'expires_at' field
        .options(
            mongodb::options::IndexOptions::builder()
                .expire_after(Some(std::time::Duration::from_secs(
                    SESSION_EXPIRATION_SECONDS,
                ))) // Auto-delete after expiration
                .build(),
        )
        .build();
    // user_session
    db.collection::<UserSessionModel>("user_sessions")
        .create_index(index.clone())
        .await
        .map_err(|e| e.to_string())?;
    // verification_token
    db.collection::<UserSessionModel>("verification_tokens")
        .create_index(index.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
