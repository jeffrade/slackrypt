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
use std::io::{stdout, Result, Write};
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
fn main() {
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
  //Notes on IV: https://security.stackexchange.com/questions/17044/when-using-aes-and-cbc-is-it-necessary-to-keep-the-iv-secret
  let key: [u8; 16] = generate_random_hex_16();
  debug!("random key is {}", String::from_utf8_lossy(&key).to_string());
  debug!("random key length is {}", &key.len());
  let iv: [u8; 16] = generate_random_hex_16();
  debug!("random iv is {}", String::from_utf8_lossy(&iv).to_string());
  debug!("random iv length is {}", &iv.len());

  type Aes128Cbc = Cbc<Aes128, Pkcs7>;
  let cipher = Aes128Cbc::new_var(&key, &iv).unwrap();
  let ciphertext: Vec<u8> = cipher.encrypt_vec(&message);

  let ciphertext_hex: String = to_hexadecimal_str(&ciphertext);
  info!("ciphertext_hex is {}", &ciphertext_hex);
  //TODO Should I then base64 encode ciphertext_hex? https://stackoverflow.com/a/44532957
  
  let ciphertext_decoded: Vec<u8> = hex::decode(&ciphertext_hex).expect("hex decoding failed!");
  assert_eq!(ciphertext, ciphertext_decoded);

  //RSA key encryption
  let cipher_vec_key: Vec<u8> = encrypt_data(&key, &public_key);
  debug!("cipher_vec_key length is {}", &cipher_vec_key.len());
  //sanity check
  let cipher_vec_key_openssl: Vec<u8> = encrypt_data(&key, &public_key_openssl);
  
  //RSA key decryption
  let private_key = get_private_key(&dir).unwrap();
  let de_key_vec: Vec<u8> = decrypt_data(&cipher_vec_key, &private_key);
  let de_key_vec_openssl: Vec<u8> = decrypt_data(&cipher_vec_key_openssl, &private_key);
  assert_eq!(&de_key_vec, &de_key_vec_openssl);
  debug!("decrypted key is {}", String::from_utf8_lossy(&de_key_vec).to_string());

  //AES message decryption
  let cipher = Aes128Cbc::new_var(&de_key_vec, &iv).unwrap();
  let mut buf: Vec<u8> = ciphertext_decoded.to_vec();
  let decrypted_ciphertext: &[u8] = cipher.decrypt(&mut buf).unwrap();
  assert_eq!(decrypted_ciphertext, message.as_slice());

  //Use OpenPGP Armor as inspiration for formatting: https://tools.ietf.org/html/rfc4880#section-6.2
  let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
  let version_header: String = String::from("Version: Slackrypt 0.1");
  let key_hex: String = to_hexadecimal_str(&cipher_vec_key);
  debug!("key_hex is {}", &key_hex);
  let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

  //Write encrypted message out to a file (and stdout)
  let file_name = String::from(&dir) + "/message.test";
  write_message_to_stdout(&begin_header, &end_header, &version_header, &ciphertext_hex, &key_hex, &iv).unwrap();
  write_message_to_file(&file_name, &begin_header, &end_header, &version_header, &ciphertext_hex, &key_hex, &iv);
  
  //Read encrypted message from a file 
  let message_from_file: String = parse_message_from_file(&file_name).unwrap();
  let file_lines: Vec<&str> = message_from_file.split("\n").collect();
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

  //RSA key decryption
  let key_hex_decoded_line: Vec<u8> = hex::decode(&key_hex_line).expect("hex decoding failed!");
  let de_key_vec_line: Vec<u8> = decrypt_data(&key_hex_decoded_line, &private_key);
  assert_eq!(de_key_vec, de_key_vec_line);
  
  //AES message decryption
  let cipher_line = Aes128Cbc::new_var(&de_key_vec_line, &iv_line.as_bytes()).unwrap();
  let ciphertext_decoded_line: Vec<u8> = hex::decode(&ciphertext_hex_line).expect("hex decoding failed!");
  assert_eq!(ciphertext_decoded, ciphertext_decoded_line);
  let mut buf_line: Vec<u8> = ciphertext_decoded_line.to_vec();
  let decrypted_ciphertext_line: &[u8] = cipher_line.decrypt(&mut buf_line).unwrap();
  assert_eq!(decrypted_ciphertext_line, message.as_slice());
  info!("decrypted message is {}", String::from_utf8_lossy(&message).to_string());
}

fn parse_message_from_file(file_name: &str) -> Result<String> {
  let mut file = File::open(file_name)?;
  let mut file_content = String::new();
  file.read_to_string(&mut file_content)?;
  Ok(file_content)
}

//A psuedo ASCII Armor format https://tools.ietf.org/html/rfc4880#section-6.2
fn write_message_to_stdout(
    begin_head: &String,
    end_head: &String,
    ver_head: &String,
    cipher: &String,
    key: &String,
    iv: &[u8]) -> Result<()> {
  let stdout = stdout();
  let mut handle = stdout.lock();
  handle.write_all(&begin_head.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(&ver_head.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(b"\n")?;
  handle.write_all(&cipher.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(&key.as_bytes())?;
  handle.write_all(b"\n")?;
  handle.write_all(&iv)?;
  handle.write_all(b"\n")?;
  //TODO the radix-64 CRC (Cyclic_redundancy_check)? =njUN
  //      -> CRC impl in C https://tools.ietf.org/html/rfc4880#section-6.1
  handle.write_all(end_head.as_bytes())?;
  handle.write_all(b"\n")?;
  Ok(())
}

fn write_message_to_file(
    file_name: &str,
    begin_head: &String,
    end_head: &String,
    ver_head: &String,
    cipher: &String,
    key: &String,
    iv: &[u8]) {
  let mut data: String = String::new();
  data.push_str(begin_head);
  data.push_str("\n");
  data.push_str(ver_head);
  data.push_str("\n");
  data.push_str("\n");
  data.push_str(cipher);
  data.push_str("\n");
  data.push_str(&key);
  data.push_str("\n");
  data.push_str(&String::from_utf8_lossy(iv));
  data.push_str("\n");
  //TODO the radix-64 CRC (Cyclic_redundancy_check)? =njUN
  //      -> CRC impl in C https://tools.ietf.org/html/rfc4880#section-6.1
  data.push_str(end_head);
  fs::write(file_name, data).expect("Unable to write encrypted message!");
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

fn to_hexadecimal_str(vec: &Vec<u8>) -> String {
  let mut hex: String = String::new();
  for i in 0..vec.len() {
    let u_8: u8 = vec[i];
    //println!("{}", &u_8);
    let v_i: String = format!("{:02x}", u_8);
    hex.push_str(&v_i);
  }
  hex
}

fn init(dir: &str) {
  simple_logger::init_by_env(); // to set, export RUST_LOG=ERROR|WARN|INFO|DEBUG

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
