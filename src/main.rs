use io::eventqueue::{Queue, QueueTrait, RawEvent};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_cors::Cors;
use actix_web::{http::header, middleware, web::Data, App, HttpServer};
use chrono::Utc;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use io::Session;

use io::utils::indexer::test_kool;
use io::{api, config::Config, DBPool};

async fn run_queue(queue: Arc<Mutex<dyn QueueTrait>>) {
    println!("Initialized queue thread.");
    loop {
        queue.lock().unwrap().update();
        tokio::time::sleep(Duration::from_millis(125)).await;
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    pretty_env_logger::init();
    let conf_path = "config.json";
    let config: Config = {
        let conf = std::fs::read_to_string(conf_path)?;
        serde_json::from_str(&conf)?
    };
    let db_string = config.db.connection_string.clone();
    let db_connections = config.db.connections;
    let cors = config.cors.clone();
    let port = config.port;
    let address = config.address.clone();
    let queue = Arc::new(Mutex::new(Queue::new()));
    let worker_queue = queue.clone();
    for folder in &config.folders.clone() {
        queue
        .lock()
        .unwrap()
        .add_event(RawEvent::FileIndexEvent { folder: folder.path.clone(), depth: folder.depth }, 0);
    }
    tokio::spawn(async move { run_queue(worker_queue).await });
    // test_kool(&config.folders.clone().into_iter().map(|f| f.path).collect());
    HttpServer::new(move || {
        let session_info = Session {
            startup: Utc::now().timestamp(),
        };
        let manager = ConnectionManager::<PgConnection>::new(&db_string);
        let pool: DBPool = Pool::builder()
            .max_size(db_connections)
            .build(manager)
            .expect("Failed to create pool.");
        App::new()
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(pool))
            .app_data(Data::new(session_info))
            .app_data(Data::new(queue.clone()))
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
    })
    .bind((address, port))?
    .run()
    .await
    
}
