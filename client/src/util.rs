use std::env;
use std::path::Path;
use std::vec::Vec;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}

pub fn to_base64_str(vec: &[u8]) -> String {
    base64::encode(vec)
}

pub fn from_base64_str(s: &str) -> Vec<u8> {
    base64::decode(s.trim()).expect("base64 decoding failed!")
}

pub fn get_user_input_message() -> Vec<u8> {
    let mut plaintext_arg: &str = "Hello World!";
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        plaintext_arg = &args[1];
    }
    let plaintext_input: String = plaintext_arg.to_string();
    let plaintext_bytes = plaintext_input.into_bytes();
    plaintext_bytes.to_vec()
}

pub fn keys_exist(key_file: &str) -> bool {
    Path::new(key_file).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_base64_str() {
        let actual_result = from_base64_str("SGVsbG8gV29ybGQhCg== ");
        assert_eq!(
            actual_result,
            vec![72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, 10]
        );
    }

    #[test]
    fn test_to_base64_str() {
        let actual_result = to_base64_str(&vec![5, 4, 3, 2, 1, 0, 42, 255]);
        assert_eq!(actual_result, "BQQDAgEAKv8=");
    }
}