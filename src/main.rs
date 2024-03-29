use actix_cors::Cors;
use actix_files::Files;
use actix_web::{http::header, middleware, web::Data, App, HttpServer};
use chrono::Utc;
use io::{
    eventqueue::{Queue, QueueTrait},
    ArcQueue,
};
use log::{debug, info};
use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use std::{env, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use utoipa_swagger_ui::SwaggerUi;
// let db: DatabaseConnection = Database::connect("protocol://username:password@host/database").await?;
use io::Session;
// use io::utils::indexer::test_kool;
use io::{api, config::Config};

use utoipa::OpenApi;

async fn run_queue(queue: Arc<Mutex<dyn QueueTrait>>) {
    info!("Initialized queue thread.");
    loop {
        queue.lock().await.update().await;
        tokio::time::sleep(Duration::from_millis(125)).await;
    }
}

async fn index() -> impl actix_web::Responder {
    actix_files::NamedFile::open_async("./static/index.html")
        .await
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(
            log::LevelFilter::from_str(
                &env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")),
            )
            .unwrap_or(log::LevelFilter::Trace),
        )
        .filter_module("sqlx::query", log::LevelFilter::Warn)
        .init();
    debug!("Initalized logger!");
    let conf_path = "config.json";
    info!("Looking for config.json in current directory.");
    let config: Config = {
        let conf = std::fs::read_to_string(conf_path)?;
        serde_json::from_str(&conf)?
    };
    let db_string = config.db.connection_string.clone();
    // let db_connections = config.db.connections;
    let cors = config.cors.clone();
    let port = config.port;
    let address = config.address.clone();
    let db: DatabaseConnection = Database::connect(db_string)
        .await
        .expect("Failed to create a database connection.");
    migration::Migrator::up(&db, None).await.unwrap();
    let queue: ArcQueue = Arc::new(Mutex::new(Queue::new(Some(db.clone()))));
    let worker_queue = queue.clone();

    // for folder in &config.folders.clone() {
    //     queue
    //     .lock()
    //     .unwrap()
    //     .add_event(RawEvent::FileIndexEvent { folder: folder.path.clone(), depth: folder.depth }, 0);
    // }
    tokio::spawn(async move { run_queue(worker_queue).await });
    // test_kool(&config.folders.clone().into_iter().map(|f| f.path).collect());
    HttpServer::new(move || {
        let session_info = Session {
            startup: Utc::now().timestamp(),
        };
        // let db: DatabaseConnection = Database::connect(&db_string)
        //     .await
        //     .expect("Failed to create a database connection.");
        App::new()
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(session_info))
            .app_data(Data::new(queue.clone()))
            .service(
                SwaggerUi::new("/swagger/{_:.*}")
                    .url("/api-doc/openapi.json", io::docs::ApiDoc::openapi()),
            )
            .service(Files::new("/content/", "./static/content"))
            .wrap({
                if let Some(cors_conf) = &cors {
                    let cors = Cors::default()
                        .supports_credentials()
                        .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "PUT"])
                        .allowed_headers(vec![
                            header::ACCEPT,
                            header::AUTHORIZATION,
                            header::CONTENT_TYPE,
                        ])
                        .max_age(3600);
                    let cors = cors_conf
                        .origins
                        .iter()
                        .fold(cors, |cors, origin| cors.allowed_origin(origin));
                    cors
                } else {
                    Cors::permissive()
                }
            })
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(api::configure)
            .configure(api::configure_no_auth)
            .default_service(actix_web::web::route().to(index))
    })
    .bind((address, port))?
    .run()
    .await
}
