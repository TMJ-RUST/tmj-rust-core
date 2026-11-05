use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{entity::UserRole, error::AppError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub jti: String,
    pub username: String,
    pub role: UserRole,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_access_token(
    user_id: &Uuid,
    username: &str,
    role: UserRole,
    jti: &str,
    secret: &str,
    expiration_secs: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::seconds(expiration_secs)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        jti: jti.to_string(),
        username: username.to_string(),
        role,
        token_type: "access".to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Token creation failed: {}", e)))
}

pub fn create_refresh_token(
    user_id: &Uuid,
    username: &str,
    role: UserRole,
    jti: &str,
    secret: &str,
    expiration_secs: i64,
) -> Result<(String, chrono::DateTime<Utc>), AppError> {
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(expiration_secs);
    let exp = expires_at.timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        jti: jti.to_string(),
        username: username.to_string(),
        role,
        token_type: "refresh".to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Token creation failed: {}", e)))?;

    Ok((token, expires_at))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
