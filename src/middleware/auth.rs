use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::{
    config::JwtAccessSecret,
    error::AppError,
    repository::SessionRepository,
    utils::token::{verify_token, Claims},
};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    JwtAccessSecret: FromRef<S>,
    SessionRepository: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header format. Expected Bearer <token>".to_string(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        let JwtAccessSecret(secret) = JwtAccessSecret::from_ref(state);

        let claims = verify_token(token, &secret)?;

        if claims.token_type != "access" {
            return Err(AppError::Unauthorized(
                "Invalid token type. Expected access token".to_string(),
            ));
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token subject".to_string()))?;

        // Verify active stateful session in Redis via SessionRepository
        let session_repo = SessionRepository::from_ref(state);
        let active_session_id = session_repo.get_active_session_id(&claims.sub).await?;

        match &active_session_id {
            Some(id) if id == &claims.jti => Ok(AuthenticatedUser { user_id, claims }),
            Some(_) => Err(AppError::Unauthorized(
                "Session superseded by another login on another device".to_string(),
            )),
            None => Err(AppError::Unauthorized(
                "Session expired or logged out".to_string(),
            )),
        }
    }
}
