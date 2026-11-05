use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};

pub async fn init_db(
    database_url: &str,
    migrator: &Migrator,
) -> Result<PgPool, Box<dyn std::error::Error>> {
    tracing::info!("Connecting to PostgreSQL database...");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to PostgreSQL: {}", e);
            e
        })?;

    tracing::info!("Running database migrations...");
    migrator.run(&db).await.map_err(|e| {
        tracing::error!("Failed to execute database migrations: {}", e);
        e
    })?;
    tracing::info!("Database migrations applied successfully.");

    Ok(db)
}
