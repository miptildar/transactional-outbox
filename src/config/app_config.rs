use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum AppEnvironment {
    Development,
    Sandbox,
    Prod,
}

impl AppEnvironment {
    pub fn as_str(&self) -> &str {
        match self {
            AppEnvironment::Development => "development",
            AppEnvironment::Sandbox => "sandbox",
            AppEnvironment::Prod => "prod",
        }
    }

    pub fn is_prod(&self) -> bool {
        self == &AppEnvironment::Prod
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = ConfigError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(AppEnvironment::Development),
            "sandbox"             => Ok(AppEnvironment::Sandbox),
            "prod" | "production" => Ok(AppEnvironment::Prod),
            other => Err(ConfigError::Message(format!(
                "Unknown APP_ENV value: '{}'. Expected: development, sandbox, prod", other
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub db_name: String,
    pub username: String,
    pub password: String,
    pub max_size: usize,
    pub max_pool_size: usize,
    pub min_pool_size: usize,
    pub connect_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub schema_registry_url: String,
    pub retries: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub postgres: PostgresConfig,
    pub kafka: KafkaConfig,
}

impl AppConfig {

    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let env = AppEnvironment::try_from(env)?;

        let config = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(File::with_name(&format!("config/{}", env.as_str())).required(false))
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true), // "true"/"false" → bool, "42" → u32
            )
            .build()?;

        let cfg: AppConfig = config.try_deserialize()?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.postgres.max_pool_size == 0 {
            return Err(ConfigError::Message(
                "postgres.max_pool_size must be > 0".into(),
            ));
        }
        if self.postgres.min_pool_size > self.postgres.max_pool_size {
            return Err(ConfigError::Message(
                "postgres.min_pool_size cannot exceed max_pool_size".into(),
            ));
        }
        if self.kafka.bootstrap_servers.is_empty() {
            return Err(ConfigError::Message(
                "kafka.bootstrap_servers cannot be empty".into(),
            ));
        }
        Ok(())
    }

    pub fn postgres_connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={} connect_timeout={}",
            self.postgres.host,
            self.postgres.port,
            self.postgres.db_name,
            self.postgres.username,
            self.postgres.password,
            self.postgres.connect_timeout_secs,
        )
    }
}

