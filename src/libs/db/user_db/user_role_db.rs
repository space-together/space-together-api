use mongodb::{bson::doc, options::IndexOptions, results::InsertOneResult, Collection, IndexModel};

use crate::{
    error::user_error::user_role_error::{UserRoleError, UserRoleResult},
    models::user_model::user_role_model::{UserRoleModel, UserRoleModelNew},
};

pub struct UserRoleDb {
    pub role: Collection<UserRoleModel>,
}

impl UserRoleDb {
    pub async fn create_user_role(
        &self,
        role: UserRoleModelNew,
    ) -> UserRoleResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "user_id" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let one_index = self.role.create_index(index).await;
        if one_index.is_err() {
            return Err(UserRoleError::RoleIsReadyExit);
        };

        todo!()
    }
}
