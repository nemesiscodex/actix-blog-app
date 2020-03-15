/// Integration Tests

use crate::config::Config;
use crate::handlers::app_config;
use actix_web::{test, App};
use deadpool_postgres::Pool;
use lazy_static::lazy_static;

/// Holds the configuration and connection pool for tests
struct TestConfig {
    _config: Config,
    pool: Pool,
}

lazy_static! {
    static ref CONFIG: TestConfig = {
        let _config = Config::from_env().unwrap();

        let pool = _config.configure_pool();

        TestConfig { _config, pool }
    };
}

#[actix_rt::test]
async fn test_health() {
    let app = App::new().data(CONFIG.pool.clone()).configure(app_config);

    let mut app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/").to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(res.status(), 200, "GET / should return 200");
}
