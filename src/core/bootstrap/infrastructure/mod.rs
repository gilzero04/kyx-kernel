use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;
use std::io;
use std::sync::Arc;

/// Loads environment variables and performs production safety checks.
pub fn load_env_and_check() -> io::Result<(u16, String, String, String)> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").expect("ENGINE_SECRET_KEY must be set");

    if env == "production" {
        if jwt_secret == "change_me_immediately_in_production" || jwt_secret.len() < 32 {
            log::error!("❌ FATAL: Weak or default JWT_SECRET detected in PRODUCTION!");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Insecure configuration in production",
            ));
        }
    }

    Ok((port, env, jwt_secret, engine_secret))
}

/// Initializes Redis connection.
pub async fn init_redis() -> io::Result<Arc<Redis>> {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let pass = std::env::var("REDIS_PASSWORD").ok();

    let redis = Redis::new(&url, pass.as_deref())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(Arc::new(redis))
}

/// Initializes Database connection.
/// NOTE: Migrations are now run separately via `./scripts/migrate-db.sh` BEFORE starting the app.
pub async fn init_database() -> io::Result<Arc<Database>> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let database = Database::new(&url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Migrations are NO LONGER run here.
    // Run `./scripts/migrate-db.sh` BEFORE starting the application.

    Ok(Arc::new(database))
}
