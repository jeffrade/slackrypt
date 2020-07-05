use std::env;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}
