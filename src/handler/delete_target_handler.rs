use sqlx::PgPool;

use crate::{
    errors::AppError,
    services::{comment_service::CommentService, like_service::LikeService},
    utils::object_id::ObjectId,
};

pub async fn delete_target_handler(pool: &PgPool, target_id: &ObjectId) -> Result<(), AppError> {
    LikeService::new(pool)
        .delete_many_by_target(target_id)
        .await?;
    CommentService::new(pool)
        .delete_many_by_target(target_id)
        .await?;
    Ok(())
}
