use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerEnvConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct PostgresEnvConfig {
    pub host: String,
    pub db_name: String,
    pub username: String,
    pub password: String,
    pub max_size: usize
}

#[derive(Debug, Deserialize)]
pub struct AppEnvConfig {
    pub server: ServerEnvConfig,
    pub postgres: PostgresEnvConfig,
}

impl AppEnvConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!(
                "config/{}",
                std::env::var("APP_ENV").unwrap_or_else(|_| "development".into())
            )).required(false))
            .add_source(Environment::with_prefix("APP").separator("__"));

        builder.build()?.try_deserialize()
    }
}

