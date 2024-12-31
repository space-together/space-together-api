use std::future;

use actix_web::{FromRequest, HttpMessage};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::{
    libs::utils::constants,
    models::{
        auth::login_model::UserLoginClaimsModel, jwt_model::user_claims_model::UserClaimsModel,
    },
};

impl FromRequest for UserClaimsModel {
    type Error = actix_web::Error;
    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> std::future::Ready<Result<UserClaimsModel, actix_web::Error>> {
        match req.extensions().get::<UserClaimsModel>() {
            Some(claim) => future::ready(Ok(claim.clone())),
            None => future::ready(Err(actix_web::error::ErrorBadRequest("Bad claim"))),
        }
    }
}

pub fn user_encode_jwt(user: UserLoginClaimsModel) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expire = Duration::hours(24);

    let claims = UserClaimsModel {
        exp: (now + expire).timestamp() as usize,
        iat: now.timestamp() as usize,
        user: UserLoginClaimsModel {
            email: user.email,
            id: user.id,
            name: user.name,
            role: user.role,
        },
    };

    let secret = (*constants::SECRET).clone();

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn user_decode_jwt(
    jwt: String,
) -> Result<TokenData<UserClaimsModel>, jsonwebtoken::errors::Error> {
    let secret = (*constants::SECRET).clone();
    let claim_data: Result<TokenData<UserClaimsModel>, jsonwebtoken::errors::Error> = decode(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );
    claim_data
}
