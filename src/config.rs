use std::sync::{Arc, LazyLock};

use proc_singleton::ArcSingleton;
use serde::{Deserialize, Serialize};

use crate::AppError;

static CONFIG: LazyLock<Arc<Config>> =
    LazyLock::new(|| Arc::new(Config::load_from_env().expect("Failed to load config")));

#[derive(Debug, Clone, Serialize, Deserialize, ArcSingleton)]
#[singleton(CONFIG)]
pub struct Config {
    pub database_url: String,
    pub api_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn load_from_env() -> Result<Config, AppError> {
        dotenvy::dotenv().map_err(|e| AppError::ConfigLoad(format!("{e:?}")))?;
        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|e| AppError::EnvVarLoad(format!("{e:?}")))?,

            api_port: std::env::var("API_PORT")
                .map_err(|e| AppError::EnvVarLoad(format!("{e:?}")))?
                .parse()
                .map_err(|e| AppError::EnvVarLoad(format!("{e:?}")))?,

            log_level: std::env::var("RUST_LOG")
                .map_err(|e| AppError::EnvVarLoad(format!("{e:?}")))?,
        };

        Ok(config)
    }
}
