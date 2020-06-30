use std::env;
use std::fs;
use std::sync::mpsc;
use std::thread;

use actix_rt::System;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    user: String,
    pubkey: String,
}

mod db;
mod slack;

// curl -H "Content-Type: application/json" --request POST --data '{"user": "ctester", "pubkey": "from curl"}' http://127.0.0.1:8080/pubkey/upload
async fn pubkey_upload(user: web::Json<User>) -> HttpResponse {
    info!("payload: {:?}", &user);
    db::insert(&user.0.user, &user.0.pubkey).unwrap();
    HttpResponse::Ok().json(user.0)
}

fn main() {
    simple_logger::init_by_env();
    init(&default_dir());
    db::init().unwrap();

    let (tx, rx) = mpsc::channel();
    let host: &str = "127.0.0.1";
    let port: &str = "8080";
    let server: String = String::from(host) + ":" + port;
    thread::spawn(move || {
        let _ = start_server(server, tx);
    });
    let _srv = rx.recv().unwrap();

    slack::init(); //This must be called last
}

// Inspiration from https://github.com/actix/examples/blob/e8ab9ee7cab3a17aedbddb4800d56d206d0a296f/run-in-thread/src/main.rs
fn start_server(server: String, tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    info!("Starting HTTP service...");
    let mut sys = System::new("slackrypt-server");

    let srv = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/pubkey/upload").route(web::post().to(pubkey_upload)))
    })
    .bind(server)?
    .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
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
