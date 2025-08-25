use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::env;

mod errors;
mod handlers;

use handlers::*;
use migration::Migrator;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    env_logger::init();

    // Database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .or_else(|_| env::var("SERVER_PORT"))
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    log::info!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .route("/documents", web::get().to(get_documents))
                    .route("/documents", web::post().to(create_document))
                    .route("/documents/{uuid}", web::get().to(get_document))
                    .route("/documents/{uuid}", web::put().to(update_document))
                    .route("/documents/{uuid}", web::delete().to(delete_document))
                    .route("/categories", web::get().to(get_categories)),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
