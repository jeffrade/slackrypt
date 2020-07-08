use std::env;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}
