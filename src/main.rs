extern crate log;
extern crate pem;
extern crate rand;
extern crate rsa;
extern crate aes_soft as aes;
extern crate block_modes;
extern crate simple_logger;
extern crate hex;

use std::convert::From;
use std::convert::Into;
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Result, Write};
use std::path::Path;
use std::process::Command;
use std::vec::Vec;

use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use log::{debug, info, warn};
use rand::rngs::OsRng;
use rsa::{PublicKey, RSAPublicKey, RSAPrivateKey, PaddingScheme};

//PKCS1 vs PKCS8 https://stackoverflow.com/questions/48958304/pkcs1-and-pkcs8-format-for-rsa-private-key
fn main() -> io::Result<()> {
  let dir = String::from(env!("HOME")) + "/.slackrypt";
  init(&dir);
  
  let private_key = get_private_key(&dir).unwrap();
  let public_key: RSAPublicKey = private_key.into();
  let public_key_openssl: RSAPublicKey = get_public_key(&dir).unwrap();
  assert_eq!(&public_key, &public_key_openssl);

  let mut message_arg: &str = "Hello World!";
  let args: Vec<String> = env::args().collect();
  if args.len() > 1 {
    message_arg = &args[1];
  }

  let message_input: String = message_arg.to_string();
  let message_bytes = message_input.into_bytes();
  let message: Vec<u8> = message_bytes.to_vec();
  
  //AES message encryption
  let key: [u8; 16] = generate_random_hex_16();
  debug!("random key is {}", String::from_utf8_lossy(&key).to_string());
  let iv: [u8; 16] = generate_random_hex_16();
  debug!("random iv is {}", String::from_utf8_lossy(&iv).to_string());

  type Aes128Cbc = Cbc<Aes128, Pkcs7>;
  let cipher = Aes128Cbc::new_var(&key, &iv).unwrap();
  let ciphertext: Vec<u8> = cipher.encrypt_vec(&message);

  //Ciphertext to hexadecimal
  let mut ciphertext_hex: String = String::new();
  for i in 0..ciphertext.len() {
    let u_8: u8 = ciphertext[i];
    let v_i: String = format!("{:02X}", u_8);//println!("{} : {:02X}", u_8, u_8);
    ciphertext_hex.push_str(&v_i);
  }
  info!("ciphertext_hex is {}", &ciphertext_hex);
  //TODO Should I then base64 encode ciphertext_hex?
  let ciphertext_decoded: Vec<u8> = hex::decode(&ciphertext_hex).expect("hex decoding failed!");
  assert_eq!(ciphertext, ciphertext_decoded);

  //RSA key(and iv) encryption
  let cipher_vec_key = encrypt_data(&key, &public_key);
  let _cipher_str_key = String::from_utf8_lossy(&cipher_vec_key);

  let cipher_vec_key_openssl = encrypt_data(&key, &public_key_openssl);
  let _cipher_str_key_openssl = String::from_utf8_lossy(&cipher_vec_key_openssl);

  //RSA key(and iv) decryption
  let private_key = get_private_key(&dir).unwrap();
  let key_vec = decrypt_data(&cipher_vec_key, &private_key);
  let key_vec_openssl = decrypt_data(&cipher_vec_key_openssl, &private_key);
  assert_eq!(&key_vec, &key_vec_openssl);
  info!("decrypted key is {}", String::from_utf8_lossy(&key_vec).to_string());

  //AES message decryption
  let cipher = Aes128Cbc::new_var(&key_vec, &iv).unwrap();
  let mut buf: Vec<u8> = ciphertext_decoded.to_vec();
  let decrypted_ciphertext: &[u8] = cipher.decrypt(&mut buf).unwrap();
  assert_eq!(decrypted_ciphertext, message.as_slice());
  info!("decrypted message is {}", String::from_utf8_lossy(&message).to_string());

  //Use PGP as inspiration for contructing a message: https://tools.ietf.org/html/rfc4880#section-6.2
  let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
  let version_header: String = String::from("Version: Slackrypt 0.1");
  let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

  //A psuedo ASCII Armor format https://tools.ietf.org/html/rfc4880#section-6.2
  let stdout = io::stdout();
  let mut handle = stdout.lock();
  handle.write_all(begin_header.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(version_header.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(b"\n")?;
  handle.write_all(ciphertext_hex.as_bytes())?;
  handle.write_all(b"\n")?;
  //TODO the key and iv? vBSFjNSiVHsuAA==
  //TODO the radix-64 CRC (Cyclic_redundancy_check)? =njUN
  //TODO  -> CRC impl in C https://tools.ietf.org/html/rfc4880#section-6.1
  handle.write_all(end_header.as_bytes())?;
  handle.write_all(b"\n")?;
  Ok(())
}

fn generate_random_hex_16() -> [u8; 16] {
  let cmd = Command::new("openssl")
    .arg("rand")
    .arg("-hex")
    .arg("16")
    .output()
    .expect("Failed to generate random hex!");
  debug!("openssl rand status {}", cmd.status);
  
  let result: &[u8] = cmd.stdout.as_slice();
  let mut ret_val = [0; 16];
  for i in 0..15 {
    ret_val[i] = result[i];
  }
  ret_val
}

fn encrypt_data(data: &[u8], public_key: &RSAPublicKey) -> Vec<u8> {
  let mut rng = OsRng;
  public_key.encrypt(&mut rng, PaddingScheme::PKCS1v15, &data[..]).expect("failed to encrypt")
}

fn decrypt_data(cipher: &[u8], private_key: &RSAPrivateKey) -> Vec<u8> {
  private_key.decrypt(PaddingScheme::PKCS1v15, &cipher).expect("failed to decrypt")
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
  debug!("openssl rsa status {}", cmd.status);
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

fn init(dir: &str) {
  simple_logger::init_by_env(); //Defaults to ERROR, set by exporting RUST_LOG

  match fs::create_dir(dir) {
    Ok(_) => true,
    Err(_) => {
      warn!("Ignore since ~/.slackrypt dir might already exist.");
      true
    }
  };

  let key_file = String::from(dir) + "/key.pem";
  if !keys_exist(&key_file) {
    create_keys(&key_file).unwrap();
  }
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

fn keys_exist(key_file: &str) -> bool {
  Path::new(key_file).exists()
}
