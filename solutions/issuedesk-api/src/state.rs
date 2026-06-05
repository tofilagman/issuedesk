use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;

/// Shared application state. Cheap to clone (pool is an Arc internally, config is
/// behind an Arc).
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        Self {
            pool,
            config: Arc::new(config),
        }
    }
}
