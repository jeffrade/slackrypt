use std::fs;
use std::sync::mpsc;
use std::thread;

use futures::executor::block_on;
use log::warn;

mod db;
mod server;
mod slack;
mod util;

fn main() {
    simple_logger::init_by_env();
    init();
    db::init().unwrap();

    let (tx, rx) = mpsc::channel();
    let server_base_url: String = util::get_env_var("SLACKRYPT_BASE_URL", "127.0.0.1:8080");
    let host_and_port: String = util::get_env_var("SLACKRYPT_HOST_AND_PORT", "127.0.0.1:8080");
    thread::spawn(move || {
        let _ = server::start_server(host_and_port, tx);
    });
    let _srv = rx.recv().unwrap();

    let _result = slack::init(&server_base_url);
    block_on(_result); // https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
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
