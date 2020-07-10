use std::sync::mpsc;

use actix_rt::System;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    user: String,
    pubkey: String,
}

// Inspiration from https://github.com/actix/examples/blob/e8ab9ee7cab3a17aedbddb4800d56d206d0a296f/run-in-thread/src/main.rs
pub fn start_server(server: String, tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    info!("Starting HTTP service...");
    let mut sys = System::new("slackrypt-server");

    //FIXME Support https https://github.com/actix/examples/tree/master/openssl
    let srv = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/pubkey/upload").route(web::post().to(pubkey_upload)))
            .service(web::resource("/pubkey/users").route(web::get().to(pubkey_users)))
    })
    .bind(server)?
    .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}

// curl -H "Content-Type: application/json" --request POST --data '{"user": "ctester", "pubkey": "from curl"}' http://127.0.0.1:8080/pubkey/upload
async fn pubkey_upload(user: web::Json<User>) -> HttpResponse {
    info!("payload: {:?}", &user);
    match db::upsert_pubkey(&user.0.user, &user.0.pubkey) {
        Ok(_) => true,
        Err(_) => {
            error!("Was not able to upsert pubkey.");
            true
        }
    };
    HttpResponse::Ok().json(user.0)
}

// curl -H "Content-Type: application/json" http://127.0.0.1:8080/pubkey/users
async fn pubkey_users() -> HttpResponse {
    debug!("pubkey_users() entering...");
    let users: Vec<String> = db::get_users_all().unwrap();
    HttpResponse::Ok().json(users)
}
