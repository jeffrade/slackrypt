use std::env;
use std::path::Path;
use std::vec::Vec;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt"
}

pub fn to_hexadecimal_str(vec: &[u8]) -> String {
    let mut hex: String = String::new();
    for u_8 in vec {
        let v_i: String = format!("{:02x}", u_8);
        hex.push_str(&v_i);
    }
    hex
}

pub fn from_hexadecimal_str(s: &str) -> Vec<u8> {
    hex::decode(s).expect("hex decoding failed!")
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
    fn test_from_hexadecimal_str() {
        let actual_result = from_hexadecimal_str("ff2a000102030405");
        assert_eq!(actual_result, vec![255, 42, 0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_to_hexadecimal_str() {
        let actual_result = to_hexadecimal_str(&vec![5, 4, 3, 2, 1, 0, 42, 255]);
        assert_eq!(actual_result, "0504030201002aff");
    }
}
