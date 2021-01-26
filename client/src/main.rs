use std::convert::From;
use std::convert::Into;
use std::fs;
use std::vec::Vec;

use rsa::RSAPublicKey;
use simple_logger::SimpleLogger;

mod crypto;
mod gui;
mod io;
mod prop;
mod util;

fn main() {
    let dir: String = util::default_dir();
    let version_header: String = String::from("Version: Slackrypt 0.3");
    init(&dir);

    let private_key = io::get_private_key(&dir).unwrap();
    let public_key: RSAPublicKey = private_key.into();
    let public_key_openssl: RSAPublicKey = io::get_public_key(&dir).unwrap();
    assert_eq!(&public_key, &public_key_openssl);

    let plaintext: Vec<u8> = util::get_user_input_message();

    //plaintext encryption
    //Notes on IV: https://security.stackexchange.com/questions/17044/when-using-aes-and-cbc-is-it-necessary-to-keep-the-iv-secret
    let key: [u8; 16] = crypto::generate_random_hex_16();
    let iv: [u8; 16] = crypto::generate_random_hex_16();
    let ciphertext: Vec<u8> = crypto::encrypt_data_sym(&key, &iv, &plaintext);

    //key encryption
    let cipher_vec_key: Vec<u8> = crypto::encrypt_data_asym(&key, &public_key);
    //sanity check
    let cipher_vec_key_openssl: Vec<u8> = crypto::encrypt_data_asym(&key, &public_key_openssl);

    //key decryption
    let private_key = io::get_private_key(&dir).unwrap();
    let de_key_vec: Vec<u8> = crypto::decrypt_data_asym(&cipher_vec_key, &private_key);
    let de_key_vec_openssl: Vec<u8> =
        crypto::decrypt_data_asym(&cipher_vec_key_openssl, &private_key);
    assert_eq!(&de_key_vec, &de_key_vec_openssl);

    //ciphertext decryption
    let decrypted_ciphertext: Vec<u8> = crypto::decrypt_sym(&de_key_vec, &iv.to_vec(), &ciphertext);
    assert_eq!(decrypted_ciphertext.as_slice(), plaintext.as_slice());

    log::info!("Starting client...");
    gui::init(&version_header); //this must be called last
}

fn init(dir: &str) {
    SimpleLogger::from_env().init().unwrap();

    match fs::create_dir(dir) {
        Ok(_) => true,
        Err(_) => {
            log::warn!("Ignore since {} dir might already exist.", dir);
            true
        }
    };

    let props = prop::get_properties();
    log::info!("Loaded properties: {:?}", &props.unwrap());

    let key_file = String::from(dir) + "/key.pem";
    if !util::keys_exist(&key_file) {
        let bits_str: String = util::get_env_var("SCRYPT_KEY_SIZE", "2048");
        let bits: i32 = bits_str.parse::<i32>().unwrap();
        crypto::create_keys_asym(bits, &key_file);
    }
}
