use std::fs;
use std::sync::mpsc;
use std::thread;

use log::warn;

mod util;

mod db;
mod server;
mod slack;

fn main() {
    simple_logger::init_by_env();
    init();
    db::init().unwrap();

    let (tx, rx) = mpsc::channel();
    let host: &str = "127.0.0.1";
    let port: &str = "8080";
    let server: String = String::from(host) + ":" + port;
    thread::spawn(move || {
        let _ = server::start_server(server, tx);
    });
    let _srv = rx.recv().unwrap();

    slack::init(); //This must be called last
}

fn init() {
    let dir: String = util::default_dir();
    match fs::create_dir(&dir) {
        Ok(_) => true,
        Err(_) => {
            warn!("Ignore since {} dir might already exist.", dir);
            true
        }
    };
}
