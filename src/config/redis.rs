use redis::aio::ConnectionManager;

pub async fn init_redis(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    tracing::info!("Connecting to Redis...");
    let redis_client = redis::Client::open(redis_url).map_err(|e| {
        tracing::error!("Failed to create Redis client: {}", e);
        e
    })?;
    let redis = ConnectionManager::new(redis_client).await.map_err(|e| {
        tracing::error!("Failed to connect to Redis: {}", e);
        e
    })?;
    tracing::info!("Redis connected successfully.");
    Ok(redis)
}
