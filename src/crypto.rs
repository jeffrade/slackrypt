use std::process::Command;
use std::vec::Vec;

use aes_soft::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use log::{debug, info};
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

pub fn encrypt_data_asym(data: &[u8], public_key: &RSAPublicKey) -> Vec<u8> {
    let mut rng = OsRng;
    public_key
        .encrypt(&mut rng, PaddingScheme::PKCS1v15, &data[..])
        .expect("failed to encrypt")
}

pub fn encrypt_data_sym(k: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_var(k, iv).unwrap();
    cipher.encrypt_vec(plaintext)
}

pub fn decrypt_data_asym(cipher: &[u8], private_key: &RSAPrivateKey) -> Vec<u8> {
    private_key
        .decrypt(PaddingScheme::PKCS1v15, &cipher)
        .expect("failed to decrypt")
}

pub fn decrypt_sym(k: &[u8], iv: &[u8], ciphertext_decoded: &[u8]) -> Vec<u8> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_var(k, iv).unwrap();
    let mut buf: Vec<u8> = ciphertext_decoded.to_vec();
    cipher.decrypt(&mut buf).unwrap().to_vec()
}

pub fn generate_random_hex_16() -> [u8; 16] {
    let cmd = Command::new("openssl")
        .arg("rand")
        .arg("-hex")
        .arg("16")
        .output()
        .expect("Failed to generate random hex!");
    debug!("openssl rand status {}", cmd.status);

    let result: &[u8] = cmd.stdout.as_slice();
    let mut ret_val = [0; 16];
    ret_val[..15].clone_from_slice(&result[..15]);
    ret_val
}

// openssl rsa -in test_key.pem -outform PEM -pubout -out test_key.pem.pub
pub fn openssl_pub_key_out(file_name: &str) {
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
    debug!(
        "openssl rsa stdout {}",
        String::from_utf8_lossy(&cmd.stdout)
    );
    debug!(
        "openssl rsa stderr: {}",
        String::from_utf8_lossy(&cmd.stderr)
    );
    chmod_file(&pub_key_file, "0644")
}

// openssl genrsa -out test_key.pem 1024
pub fn openssl_generate(file_name: &str, bits: i32) {
    let cmd = Command::new("openssl")
        .arg("genrsa")
        .arg("-out")
        .arg(file_name)
        .arg(format!("{}", bits))
        .output()
        .expect("Failed to generate keys!");
    debug!("openssl genrsa returned {}", cmd.status);
}

pub fn create_keys_asym(bits: i32, key_file: &str) {
    // let bits_str = String::from(env!("SCRYPT_KEY_SIZE")); //Set this to min of 2048
    // let bits: i32 = bits_str.parse::<i32>().unwrap();
    info!("Creating {} bit keys, this may take a while...", bits);

    // Using openssl since RustCrypto/RSA cannot export keys in PEM.
    // See issue https://github.com/RustCrypto/RSA/issues/31
    openssl_generate(key_file, bits);
    chmod_file(key_file, "0400");
    openssl_pub_key_out(key_file);
}

fn chmod_file(file_name: &str, permissions: &str) {
    let cmd = Command::new("chmod")
        .arg(permissions)
        .arg(file_name)
        .output()
        .expect("Failed to chmod file!");
    debug!("chmod cmd returned {}", cmd.status);
}
