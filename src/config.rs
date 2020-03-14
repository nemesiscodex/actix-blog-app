pub use config::ConfigError;
use deadpool_postgres::Pool;
use dotenv::dotenv;
use serde::Deserialize;
use slog::{o, Drain};
use slog_async;
use slog_envlogger;
use slog_term;
use tokio_postgres::NoTls;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
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
