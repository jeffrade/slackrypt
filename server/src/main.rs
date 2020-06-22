use std::env;
use std::fs;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    user: String,
    pubkey: String,
}

mod db;

// curl -H "Content-Type: application/json" --request POST --data '{"user": "ctester", "pubkey": "from curl"}' http://127.0.0.1:8080/pubkey/upload
async fn pubkey_upload(user: web::Json<User>) -> HttpResponse {
    info!("payload: {:?}", &user);
    db::insert(&user.0.user, &user.0.pubkey).unwrap();
    HttpResponse::Ok().json(user.0) // <- send response
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_by_env();
    init(&default_dir());
    db::init().unwrap();
    let host: &str = "127.0.0.1";
    let port: &str = "8080";
    let server: String = String::from(host) + ":" + port;
    info!("Starting slackrypt-server...");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/pubkey/upload").route(web::post().to(pubkey_upload)))
    })
    .bind(&server)?
    .run()
    .await
}

fn init(dir: &str) {
    match fs::create_dir(dir) {
        Ok(_) => true,
        Err(_) => {
            warn!("Ignore since {} dir might already exist.", dir);
            true
        }
    };
}

fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}
