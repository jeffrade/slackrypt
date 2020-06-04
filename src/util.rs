use std::env;
use std::path::Path;
use std::vec::Vec;

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
