mod auth;
mod config;
mod db;
mod dto;
mod error;
mod handlers;
mod models;
mod router;
mod state;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::AppConfig, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env in dev; real env vars take precedence in prod.
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    tracing::info!(port = config.port, "starting issuedesk-api");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.database_url)
        .await?;

    // Run migrations on boot.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Ensure the upload dir exists.
    tokio::fs::create_dir_all(&config.upload_dir).await.ok();

    // Seed an admin user when the table is empty.
    seed_admin(&pool, &config).await?;

    let port = config.port;
    let state = AppState::new(pool, config);
    let app = router::build(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn seed_admin(pool: &sqlx::PgPool, config: &AppConfig) -> anyhow::Result<()> {
    if !config.seed_admin.enabled {
        return Ok(());
    }
    if db::users::count(pool).await? > 0 {
        return Ok(());
    }
    let hash = auth::password::hash_password(&config.seed_admin.password)
        .map_err(|e| anyhow::anyhow!("seed hash failed: {e}"))?;
    db::users::create(
        pool,
        &config.seed_admin.username,
        &config.seed_admin.email,
        "Administrator",
        &hash,
        crate::models::enums::Role::Admin.as_i16(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("seed admin failed: {e}"))?;
    tracing::warn!(
        username = %config.seed_admin.username,
        "seeded initial admin user (change the password!)"
    );
    Ok(())
}
