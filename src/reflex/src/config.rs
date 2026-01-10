use std::env;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Config {
    pub kraken_api_key: String,
    pub kraken_secret: String,
    pub redis_url: String,
    pub database_url: String,
    pub questdb_host: String,
    pub questdb_ilp_port: String,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let kraken_api_key = env::var("KRAKEN_API_KEY").unwrap_or_default();
        let kraken_secret = env::var("KRAKEN_SECRET").unwrap_or_default();

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://admin:quest@localhost:8812/qdb".to_string());

        let questdb_host = env::var("QUESTDB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());

        let questdb_ilp_port = env::var("QUESTDB_ILP_PORT")
            .unwrap_or_else(|_| "9009".to_string());

        Ok(Self {
            kraken_api_key,
            kraken_secret,
            redis_url,
            database_url,
            questdb_host,
            questdb_ilp_port,
        })
    }
}
