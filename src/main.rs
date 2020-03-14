mod config;
mod handlers;
mod models;
mod errors;

use crate::config::Config;
use crate::handlers::app_config;
use actix_web::{middleware, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env().unwrap();

    let pool = config.configure_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .configure(app_config)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

#[cfg(test)]
mod integration_tests;
