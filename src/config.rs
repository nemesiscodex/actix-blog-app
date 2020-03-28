pub use config::ConfigError;
use crate::errors::{AppError, AppErrorType};
use deadpool_postgres::Pool;
use dotenv::dotenv;
use serde::Deserialize;
use slog::{o, Drain};
use slog_async;
use slog_envlogger;
use slog_term;
use tokio_postgres::NoTls;
use argonautica::Hasher;
use futures::compat::Future01CompatExt;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
    pub url: String,
    pub secret_key: String
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::load_dotenv();
        Self::configure_log();

        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new().separator("__"))?;
        cfg.try_into()
    }

    fn load_dotenv() {
        dotenv().ok();
    }

    pub fn configure_pool(&self) -> Pool {
        self.pg.create_pool(NoTls).unwrap()
    }

    pub fn hashing_service(&self) -> HashingService {
        HashingService {
            secret_key: self.server.secret_key.clone()
        }
    }

    fn configure_log() {
        let decorator = slog_term::TermDecorator::new().build();
        let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
        let console_drain = slog_envlogger::new(console_drain);
        let console_drain = slog_async::Async::new(console_drain).build().fuse();
        let log = slog::Logger::root(console_drain, o!("v" => env!("CARGO_PKG_VERSION")));
        slog_scope::set_global_logger(log).cancel_reset();
        slog_stdlog::init().ok();
    }
}

#[derive(Clone)]
pub struct HashingService {
    secret_key: String
}

impl HashingService {
    pub async fn hash(&self, password: String) -> Result<String, AppError> { // ~300ms
        Hasher::default()
            .with_password(&password)
            .with_secret_key(&self.secret_key)
            .hash_non_blocking()
            .compat()
            .await
            .map_err(|err| {
                AppError {
                    message: Some("Invalid password provided".to_string()),
                    cause: Some(err.to_string()),
                    error_type: AppErrorType::InvalidField
                }
            })
    }
}