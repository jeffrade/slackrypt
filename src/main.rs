use std::convert::From;
use std::convert::Into;
use std::env;
use std::fs;
use std::vec::Vec;

use log::{debug, info, warn};
use rsa::RSAPublicKey;

mod crypto;
mod gui;
mod io;
mod prop;
mod util;

//PKCS1 vs PKCS8 https://stackoverflow.com/questions/48958304/pkcs1-and-pkcs8-format-for-rsa-private-key
fn main() {
    //TODO A lot of assert'ing going on here. Anything test-like, give it a new home.

    let dir: String = util::default_dir();
    init(&dir);

    let private_key = io::get_private_key(&dir).unwrap();
    let public_key: RSAPublicKey = private_key.into();
    let public_key_openssl: RSAPublicKey = io::get_public_key(&dir).unwrap();
    assert_eq!(&public_key, &public_key_openssl);

    info!("\n{}", io::get_public_key_string(&dir).unwrap());

    let plaintext: Vec<u8> = util::get_user_input_message();

    //plaintext encryption
    //Notes on IV: https://security.stackexchange.com/questions/17044/when-using-aes-and-cbc-is-it-necessary-to-keep-the-iv-secret
    let key: [u8; 16] = crypto::generate_random_hex_16();
    debug!(
        "random key is {}",
        String::from_utf8_lossy(&key).to_string()
    );
    debug!("random key length is {}", &key.len());
    let iv: [u8; 16] = crypto::generate_random_hex_16();
    debug!("random iv is {}", String::from_utf8_lossy(&iv).to_string());
    debug!("random iv length is {}", &iv.len());

    let ciphertext: Vec<u8> = crypto::encrypt_data_sym(&key, &iv, &plaintext);
    let ciphertext_hex: String = util::to_hexadecimal_str(&ciphertext);
    info!("ciphertext_hex is {}", &ciphertext_hex);
    //TODO Should I then base64 encode ciphertext_hex? https://stackoverflow.com/a/44532957
    //     Or more easily, just go from Vec<u8> to base64? https://stackoverflow.com/a/58051911

    let ciphertext_decoded: Vec<u8> = util::from_hexadecimal_str(&ciphertext_hex);
    assert_eq!(ciphertext, ciphertext_decoded);

    //key encryption
    let cipher_vec_key: Vec<u8> = crypto::encrypt_data_asym(&key, &public_key);
    debug!("cipher_vec_key length is {}", &cipher_vec_key.len());
    //sanity check
    let cipher_vec_key_openssl: Vec<u8> = crypto::encrypt_data_asym(&key, &public_key_openssl);

    //key decryption
    let private_key = io::get_private_key(&dir).unwrap();
    let de_key_vec: Vec<u8> = crypto::decrypt_data_asym(&cipher_vec_key, &private_key);
    let de_key_vec_openssl: Vec<u8> =
        crypto::decrypt_data_asym(&cipher_vec_key_openssl, &private_key);
    assert_eq!(&de_key_vec, &de_key_vec_openssl);
    debug!(
        "decrypted key is {}",
        String::from_utf8_lossy(&de_key_vec).to_string()
    );

    //ciphertext decryption
    let decrypted_ciphertext: Vec<u8> =
        crypto::decrypt_sym(&de_key_vec, &iv.to_vec(), &ciphertext_decoded);
    assert_eq!(decrypted_ciphertext.as_slice(), plaintext.as_slice());

    let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
    let version_header: String = String::from("Version: Slackrypt 0.1");
    let key_hex: String = util::to_hexadecimal_str(&cipher_vec_key);
    debug!("key_hex is {}", &key_hex);
    let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

    //Write encrypted message out to a file (and stdout)
    let file_name = String::from(&dir) + "/message.test";
    io::write_message_to_stdout(
        &begin_header,
        &end_header,
        &version_header,
        &ciphertext_hex,
        &key_hex,
        &iv,
    )
    .unwrap();
    io::write_message_to_file(
        &file_name,
        &begin_header,
        &end_header,
        &version_header,
        &ciphertext_hex,
        &key_hex,
        &iv,
    );

    //Read encrypted message from a file
    let message_from_file: String = io::parse_message_from_file(&file_name).unwrap();
    let file_lines: Vec<&str> = message_from_file.split('\n').collect();
    let version_header_line: &str = file_lines[1];
    assert_eq!(&version_header, &version_header_line);
    let blank_line: &str = file_lines[2];
    assert_eq!("", blank_line);
    let ciphertext_hex_line: &str = file_lines[3];
    assert_eq!(ciphertext_hex, ciphertext_hex_line);
    let key_hex_line: &str = file_lines[4];
    assert_eq!(key_hex, key_hex_line);
    let iv_line: &str = file_lines[5];
    assert_eq!(&String::from_utf8_lossy(&iv), iv_line);

    //key decryption
    let key_hex_decoded_line: Vec<u8> = util::from_hexadecimal_str(&key_hex_line);
    let de_key_vec_line: Vec<u8> = crypto::decrypt_data_asym(&key_hex_decoded_line, &private_key);
    assert_eq!(de_key_vec, de_key_vec_line);

    //ciphertext decryption
    let ciphertext_decoded_line: Vec<u8> = util::from_hexadecimal_str(&ciphertext_hex_line);
    assert_eq!(ciphertext_decoded, ciphertext_decoded_line);
    let decrypted_ciphertext_line: Vec<u8> = crypto::decrypt_sym(
        &de_key_vec_line,
        &iv_line.as_bytes().to_vec(),
        &ciphertext_decoded_line,
    );
    assert_eq!(decrypted_ciphertext_line.as_slice(), plaintext.as_slice());
    info!(
        "decrypted ciphertext is {}",
        String::from_utf8_lossy(decrypted_ciphertext_line.as_slice()).to_string()
    );
    gui::init(&version_header);
}

fn init(dir: &str) {
    simple_logger::init_by_env(); // to set, export RUST_LOG=ERROR|WARN|INFO|DEBUG

    match fs::create_dir(dir) {
        Ok(_) => true,
        Err(_) => {
            warn!("Ignore since {} dir might already exist.", dir);
            true
        }
    };

    let props = prop::get_properties();
    info!("Loaded properties: {:?}", &props.unwrap());

    let key_file = String::from(dir) + "/key.pem";
    if !util::keys_exist(&key_file) {
        let bits_str = String::from(env!("SCRYPT_KEY_SIZE")); //Set this to min of 2048
        let bits: i32 = bits_str.parse::<i32>().unwrap();
        crypto::create_keys_asym(bits, &key_file);
    }
}
