use std::convert::TryFrom;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}

pub fn generate_rand() -> u128 {
    rand::random::<u128>()
}

pub fn get_current_time() -> Option<u32> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get current time");
    let sec = since_the_epoch.as_secs();
    u32::try_from(sec).ok()
}

pub fn get_current_time_plus(seconds: u32) -> u32 {
    get_current_time().unwrap() + seconds
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_generate_rand() {
        let r: u128 = generate_rand();
        assert_eq!(mem::size_of_val(&r), 16);
    }
}
