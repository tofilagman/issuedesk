use std::env;

/// Application configuration, sourced entirely from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_ttl_hours: i64,
    pub upload_dir: String,
    pub max_upload_bytes: usize,
    pub static_dir: String,
    pub seed_admin: SeedAdmin,
}

#[derive(Debug, Clone)]
pub struct SeedAdmin {
    pub enabled: bool,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        // DATABASE_URL wins if explicitly set (dev convenience); otherwise compose
        // it from the APP_* parts the way the other Outsource projects do.
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                let server = var_or("APP_SERVER", "localhost");
                let port = var_or("APP_PORT", "5432");
                let db = var_or("APP_DATABASE", "issuedeskdevl");
                let user = var_or("APP_USER", "postgres");
                let pass = var_or("APP_PASSWORD", "postgres");
                format!("postgres://{user}:{pass}@{server}:{port}/{db}")
            }
        };

        Ok(Self {
            database_url,
            port: var_or("PORT", "8080").parse().unwrap_or(8080),
            jwt_secret: var_or("JWT_SECRET", "change-me-in-production"),
            jwt_ttl_hours: var_or("JWT_TTL_HOURS", "24").parse().unwrap_or(24),
            upload_dir: var_or("UPLOAD_DIR", "./data/uploads"),
            max_upload_bytes: var_or("MAX_UPLOAD_BYTES", "26214400")
                .parse()
                .unwrap_or(26_214_400),
            static_dir: var_or("STATIC_DIR", "./static"),
            seed_admin: SeedAdmin {
                enabled: var_or("SEED_ADMIN", "false").eq_ignore_ascii_case("true"),
                username: var_or("SEED_ADMIN_USERNAME", "admin"),
                email: var_or("SEED_ADMIN_EMAIL", "admin@issuedesk.local"),
                password: var_or("SEED_ADMIN_PASSWORD", "admin123"),
            },
        })
    }
}

fn var_or(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}
