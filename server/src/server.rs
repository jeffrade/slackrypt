use std::sync::mpsc;

use actix_rt::System;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::util;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    user: String,
    pubkey: String,
}

// Inspiration from https://github.com/actix/examples/blob/e8ab9ee7cab3a17aedbddb4800d56d206d0a296f/run-in-thread/src/main.rs
pub fn start_server(host_and_port: String, tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    log::info!("Starting HTTP service...");
    let mut sys = System::new("slackrypt-server");

    let srv = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/pubkey/users").route(web::get().to(pubkey_users)))
            .service(web::resource("/init.sh").route(web::get().to(init_user)))
    })
    .bind(&host_and_port)?
    .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}

// curl -H "Content-Type: application/json" http://127.0.0.1:8080/pubkey/users
async fn pubkey_users() -> HttpResponse {
    log::debug!("pubkey_users() entering...");
    let users: Vec<String> = db::get_users_all().unwrap();
    HttpResponse::Ok().json(users)
}

// curl -H "Content-Type: text/plain" http://127.0.0.1:8080/init.sh
async fn init_user() -> HttpResponse {
    log::debug!("init_user() entering...");
    let server_base_url: String = util::get_env_var("SLACKRYPT_BASE_URL", "127.0.0.1:8080");
    let response: String = util::get_init_sh_cmd(format!("https://{}", &server_base_url).as_str());
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(&response)
}
