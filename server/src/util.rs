use std::env;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}

pub fn get_host() -> String {
    match env::var("SLACKRYPT_HOST") {
        Ok(value) => value,
        Err(_e) => String::from("127.0.0.1"),
    }
}

pub fn get_port() -> String {
    match env::var("SLACKRYPT_PORT") {
        Ok(value) => value,
        Err(_e) => String::from("8080"),
    }
}
