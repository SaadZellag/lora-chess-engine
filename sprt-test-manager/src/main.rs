mod db;
mod models;
mod handlers;
mod engine;
mod fastchess;
mod discord;
mod config;
mod errors;

use actix_web::{web, App, HttpServer, middleware};
use std::sync::Arc;
use parking_lot::Mutex;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv::dotenv().ok();

    info!("Initializing SPRT Test Manager...");

    // Initialize database
    let db = db::Database::new("sprt_tests.db")
        .expect("Failed to initialize database");
    let db = Arc::new(Mutex::new(db));

    // Load config
    let config = config::Config::load()
        .expect("Failed to load config");
    let config = Arc::new(config);

    info!("Starting server on 0.0.0.0:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Logger::default())
            .wrap(actix_web::middleware::NormalizePath::trim())
            // API routes
            .route("/api/tests", web::get().to(handlers::tests::list_tests))
            .route("/api/tests", web::post().to(handlers::tests::create_test))
            .route("/api/tests/{id}", web::get().to(handlers::tests::get_test))
            .route("/api/tests/{id}", web::delete().to(handlers::tests::delete_test))
            .route("/api/tests/{id}/pause", web::post().to(handlers::tests::pause_test))
            .route("/api/tests/{id}/resume", web::post().to(handlers::tests::resume_test))
            .route("/api/tests/{id}/logs", web::get().to(handlers::tests::get_logs))
            .route("/api/settings", web::get().to(handlers::settings::get_settings))
            .route("/api/settings", web::put().to(handlers::settings::update_settings))
            // Static files
            .route("/", web::get().to(handlers::static_files::index))
            .route("/{tail:.*}", web::get().to(handlers::static_files::static_file))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
