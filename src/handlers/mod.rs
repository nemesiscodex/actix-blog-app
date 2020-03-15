use actix_web::{web, HttpResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn app_config(config: &mut web::ServiceConfig) {
    config
        .service(web::resource("/").route(web::get().to(health)));
}
