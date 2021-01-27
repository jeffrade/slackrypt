#![feature(decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use std::fs;
use std::thread;

use simple_logger::SimpleLogger;

mod db;
mod server;
mod slack;
mod util;

fn main() {
    SimpleLogger::from_env().init().unwrap();
    init();
    start_services();
}

fn init() {
    let dir: String = util::default_dir();
    match fs::create_dir(&dir) {
        Ok(_) => true,
        Err(_) => {
            log::warn!("Ignore since {} dir might already exist.", dir);
            true
        }
    };
}

fn start_services() {
    db::init().expect("Could not initialize and start the database!");
    start_slack_bot();
    server::start_server();
}

fn start_slack_bot() {
    let server_base_url: String = util::get_env_var("SLACKRYPT_BASE_URL", "127.0.0.1:8000");
    thread::spawn(move || {
        slack::init(&server_base_url);
    });
}
