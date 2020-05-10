extern crate log;
extern crate pem;
extern crate rand;
extern crate rsa;
extern crate simple_logger;

use std::convert::From;
use std::convert::Into;
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;
use std::process::Command;
use std::vec::Vec;

use log::{debug, info, warn};
use rand::rngs::OsRng;
use rsa::{PublicKey, RSAPublicKey, RSAPrivateKey, PaddingScheme};

//PKCS1 vs PKCS8 https://stackoverflow.com/questions/48958304/pkcs1-and-pkcs8-format-for-rsa-private-key
fn main() {  
  simple_logger::init_by_env(); //Defaults to ERROR, set by exporting RUST_LOG

  let dir = String::from(env!("HOME")) + "/.slackrypt";
  let key_file = String::from(&dir) + "/key.pem";

  match fs::create_dir(&dir) {
    Ok(_) => true,
    Err(_) => {
      warn!("Does ~/.slackrypt dir exist? Skipping for now...");
      true
    }
  };

  if !keys_exist(&key_file) {
    create_keys(&key_file).unwrap();
  }

  let private_key = get_private_key(&dir).unwrap();
  let public_key: RSAPublicKey = private_key.into();
  let public_key_openssl: RSAPublicKey = get_public_key(&dir).unwrap();
  assert_eq!(&public_key, &public_key_openssl);

  let mut message: &str = "Hello World!";
  let args: Vec<String> = env::args().collect();
  if args.len() > 1 {
    message = &args[1];
  }

  let message_input: String = message.to_string();
  let message_bytes = message_input.into_bytes();
  let message: Vec<u8> = message_bytes.to_vec();
  let cipher_vec = encrypt_message(&message, &public_key);
  let _cipher_str = String::from_utf8_lossy(&cipher_vec);

  let cipher_vec_openssl = encrypt_message(&message, &public_key_openssl);
  let _cipher_str_openssl = String::from_utf8_lossy(&cipher_vec_openssl);

  let private_key = get_private_key(&dir).unwrap();
  let message_str = decrypt_message(&cipher_vec, &private_key);
  let message_str_openssl = decrypt_message(&cipher_vec_openssl, &private_key);

  assert_eq!(&message_str, &message_str_openssl);
  info!("decrypted message is {}", &message_str);
}

fn encrypt_message(data: &[u8], public_key: &RSAPublicKey) -> Vec<u8> {
  let mut rng = OsRng;
  public_key.encrypt(&mut rng, PaddingScheme::PKCS1v15, &data[..]).expect("failed to encrypt")
}

fn decrypt_message(cipher: &[u8], private_key: &RSAPrivateKey) -> String {
  let message_vec = private_key.decrypt(PaddingScheme::PKCS1v15, &cipher).expect("failed to decrypt");
  String::from_utf8_lossy(&message_vec).to_string()
}

fn get_public_key(dir: &str) -> Result<RSAPublicKey> {
  let file_name = String::from(dir) + "/key.pem.pub";
  let mut file = File::open(file_name)?;
  let mut file_content = String::new();
  file.read_to_string(&mut file_content)?;
  let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
  let public_key = RSAPublicKey::try_from(pem_encoded).expect("failed to parse key");
  Ok(public_key)
}

fn get_private_key(dir: &str) -> Result<RSAPrivateKey> {
  let file_name = String::from(dir) + "/key.pem";
  let mut file = File::open(file_name)?;
  let mut file_content = String::new();
  file.read_to_string(&mut file_content)?;
  let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
  let private_key = RSAPrivateKey::try_from(pem_encoded).expect("failed to parse key");
  Ok(private_key)
}

fn keys_exist(key_file: &str) -> bool {
  Path::new(key_file).exists()
}

fn create_keys(key_file: &str) -> Result<()> {
  let bits_str = String::from(env!("SCRYPT_KEY_SIZE")); //Set this to min of 2048
  let bits: i32 = bits_str.parse::<i32>().unwrap();
  info!("Creating {} bit keys, this may take a while...", bits);

  // Using openssl since RustCrypto/RSA cannot export keys in PEM.
  // See issue https://github.com/RustCrypto/RSA/issues/31
  openssl_generate(key_file, bits);
  chmod_file(key_file, "0400");
  openssl_pub_key_out(key_file);
  Ok(())
}

// openssl rsa -in test_key.pem -outform PEM -pubout -out test_key.pem.pub
fn openssl_pub_key_out(file_name: &str) {
  let mut pub_key_file = String::from(file_name);
  pub_key_file.push_str(".pub");
  let cmd = Command::new("openssl")
    .arg("rsa")
    .arg("-in")
    .arg(file_name)
    .arg("-outform")
    .arg("PEM")
    .arg("-pubout")
    .arg("-out")
    .arg(&pub_key_file)
    .output()
    .expect("Failed to generate keys!");
  info!("openssl rsa status {}", cmd.status);
  debug!("openssl rsa stdout {}", String::from_utf8_lossy(&cmd.stdout));
  debug!("openssl rsa stderr: {}", String::from_utf8_lossy(&cmd.stderr));
  chmod_file(&pub_key_file, "0644")
}

// openssl genrsa -out test_key.pem 1024
fn openssl_generate(file_name: &str, bits: i32) {
  let cmd = Command::new("openssl")
    .arg("genrsa")
    .arg("-out")
    .arg(file_name)
    .arg(format!("{}", bits))
    .output()
    .expect("Failed to generate keys!");
  debug!("openssl genrsa returned {}", cmd.status);
}

fn chmod_file(file_name: &str, permissions: &str) {
  let cmd = Command::new("chmod")
    .arg(permissions)
    .arg(file_name)
    .output()
    .expect("Failed to chmod file!");
  debug!("chmod cmd returned {}", cmd.status);
}
