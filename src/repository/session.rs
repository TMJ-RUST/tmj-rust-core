use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

use crate::{entity::UserRole, error::AppError};

#[derive(Clone)]
pub struct SessionRepository {
    redis: ConnectionManager,
}

impl SessionRepository {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn save_session(
        &self,
        user_id: &Uuid,
        username: &str,
        role: UserRole,
        session_id: &str,
        refresh_token: &str,
        ttl_secs: i64,
    ) -> Result<(), AppError> {
        let session_key = format!("session:user:{}", user_id);
        let mut redis = self.redis.clone();

        redis::pipe()
            .atomic()
            .hset(&session_key, "session_id", session_id)
            .hset(&session_key, "refresh_token", refresh_token)
            .hset(&session_key, "user_id", user_id.to_string())
            .hset(&session_key, "username", username)
            .hset(&session_key, "role", role.to_string())
            .expire(&session_key, ttl_secs)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to save session in Redis: {}", e))
            })
    }

    pub async fn get_active_session_id(&self, user_sub: &str) -> Result<Option<String>, AppError> {
        let session_key = format!("session:user:{}", user_sub);
        let mut redis = self.redis.clone();

        redis
            .hget(&session_key, "session_id")
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis session lookup failed: {}", e)))
    }

    pub async fn get_session_tokens(
        &self,
        user_id: &Uuid,
    ) -> Result<(Option<String>, Option<String>), AppError> {
        let session_key = format!("session:user:{}", user_id);
        let mut redis = self.redis.clone();

        let active_session_id: Option<String> = redis
            .hget(&session_key, "session_id")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Redis session lookup failed: {}", e))
            })?;

        let stored_refresh_token: Option<String> = redis
            .hget(&session_key, "refresh_token")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Redis session lookup failed: {}", e))
            })?;

        Ok((active_session_id, stored_refresh_token))
    }

    pub async fn delete_session(&self, user_sub: &str) -> Result<(), AppError> {
        let session_key = format!("session:user:{}", user_sub);
        let mut redis = self.redis.clone();

        redis.del(&session_key).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete session in Redis: {}", e))
        })
    }

    pub async fn ping(&self) -> Result<(), redis::RedisError> {
        let mut redis = self.redis.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut redis)
            .await
            .map(|_| ())
    }
}
