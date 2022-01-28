use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{http::header, middleware, web::Data, App, HttpServer};

use io::{config::Config, api};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    pretty_env_logger::init();
    let conf_path = "config.json";
    let config: Config = {
        let conf = std::fs::read_to_string(conf_path)?;
        serde_json::from_str(&conf)?
    };
    let cors = config.cors.clone();
    let port = config.port;
    let address = config.address.clone();
    let conf = Arc::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&conf)))
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
    })
    .bind((address, port))?
    .run()
    .await
}
