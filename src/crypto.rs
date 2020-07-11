use std::process::Command;
use std::vec::Vec;

use aes_soft::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use log::{debug, info};
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

use crate::io;
use crate::util;

pub fn slackrypt(plaintext: &[u8], public_key: &RSAPublicKey) -> String {
    let key: [u8; 16] = generate_random_hex_16();
    let cipher_vec_key: Vec<u8> = encrypt_data_asym(&key, public_key);
    let key_hex: String = util::to_hexadecimal_str(&cipher_vec_key);
    let iv: [u8; 16] = generate_random_hex_16();
    let ciphertext: Vec<u8> = encrypt_data_sym(&key, &iv, &plaintext);
    let ciphertext_hex: String = util::to_hexadecimal_str(&ciphertext);

    let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
    let version_header: String = String::from("Version: Slackrypt 0.1");
    let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

    io::build_armor_message(
        &begin_header,
        &end_header,
        &version_header,
        &ciphertext_hex,
        &key_hex,
        &iv,
    )
}

pub fn unslackrypt(armor: &str) -> String {
    let private_key: RSAPrivateKey = io::get_private_key_default().unwrap();
    unslackrypt_with_key(armor, &private_key)
}

pub fn unslackrypt_with_key(armor: &str, private_key: &RSAPrivateKey) -> String {
    let file_lines: Vec<&str> = armor.split('\n').collect();
    let ciphertext_hex_line: &str = file_lines[3];
    let ciphertext: Vec<u8> = util::from_hexadecimal_str(&ciphertext_hex_line);
    let key_hex_line: &str = file_lines[4];
    let key_hex_decoded_line: Vec<u8> = util::from_hexadecimal_str(&key_hex_line);
    let key: Vec<u8> = decrypt_data_asym(&key_hex_decoded_line, &private_key);
    let iv_line: &str = file_lines[5];
    let iv = iv_line.as_bytes().to_vec();
    let byte_vec: Vec<u8> = decrypt_sym(&key, &iv, &ciphertext);
    String::from_utf8_lossy(&byte_vec).to_string()
}

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

pub fn decrypt_sym(k: &[u8], iv: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_var(k, iv).unwrap();
    let mut buf: Vec<u8> = ciphertext.to_vec();
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
    ret_val[..16].clone_from_slice(&result[..16]);
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

//PKCS1 vs PKCS8 https://stackoverflow.com/questions/48958304/pkcs1-and-pkcs8-format-for-rsa-private-key
// openssl genrsa -out test_key.pem 2048
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

#[cfg(test)]
mod tests {
    use super::*;
    use pem;
    use std::convert::TryFrom;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::Result;

    #[test]
    fn test_slackrypt() {
        let private_key: RSAPrivateKey = read_private_key().unwrap();
        let public_key: RSAPublicKey = read_public_key().unwrap();
        let armor_msg: String = slackrypt("Hello World!".as_bytes(), &public_key);
        let file_lines: Vec<&str> = armor_msg.split('\n').collect();
        assert_eq!("-----BEGIN SLACKRYPT MESSAGE-----", file_lines[0]);
        assert_eq!("Version: Slackrypt 0.1", file_lines[1]);
        assert_eq!("", file_lines[2]);
        assert_eq!("-----END SLACKRYPT MESSAGE-----", file_lines[6]);

        let ciphertext_hex_line: &str = file_lines[3];
        let ciphertext: Vec<u8> = util::from_hexadecimal_str(&ciphertext_hex_line);
        let key_hex_line: &str = file_lines[4];
        let encrypted_key: Vec<u8> = util::from_hexadecimal_str(&key_hex_line);
        let key: Vec<u8> = decrypt_data_asym(&encrypted_key, &private_key);
        let iv_line: &str = file_lines[5];

        let actual_plaintext: Vec<u8> =
            decrypt_sym(&key, &iv_line.as_bytes().to_vec(), &ciphertext);
        assert_eq!(actual_plaintext.as_slice(), "Hello World!".as_bytes());
    }

    #[test]
    pub fn test_unslackrypt() {
        let armor_msg = "-----BEGIN SLACKRYPT MESSAGE-----\nVersion: Slackrypt 0.1\n\n5c5d6d192aec965914f1c2ad7380a6c3\n4b7282437d51b745e43a38e90c5b7b3f044ad5d4fc90bc6b666153be69a532bb3dd7a59301494cc487ab78188922326fadcdd986cb21ee98e96181ad1c9214888f06ac6c73d8a5cbab154836f108297884dabdcc048823898f1b045e3bd3e13e94b607431f6267073ac852ae9335aed3f5a12bdd71b6e3f30427553459651e8e93c30e53136480229f2dc8189019b06f5dfec610181101cdb72856c479fa132ce8cd5950defc8c7af3fc8fe7806a5b10538a42792734bce9dc48bcb6d2802d21fb3c95e199fc9b57d734e3f3359576caf23309eea8761938b3becd0ec1a9ee17a2a7e29db843f23b8f3d99d6653beeddb9d91443c683bca58a41aa12b929b2eb\n1e357ddda8d80d7f\n-----END SLACKRYPT MESSAGE-----";
        let private_key: RSAPrivateKey = read_private_key().unwrap();
        let plaintext = unslackrypt_with_key(armor_msg, &private_key);
        assert_eq!("Hello World!".as_bytes(), plaintext.as_bytes());
    }

    #[test]
    fn test_encrypt_data_asym() {
        let data: Vec<u8> = b"Hello world. This is my plaintext message".to_vec();
        let private_key: RSAPrivateKey = read_private_key().unwrap();
        let public_key: RSAPublicKey = private_key.into();
        let expected_public_key: RSAPublicKey = read_public_key().unwrap();
        assert_eq!(public_key, expected_public_key);
        let actual: Vec<u8> = encrypt_data_asym(&data, &public_key);
        assert_eq!(actual.len(), 256);
    }

    #[test]
    fn test_encrypt_data_sym() {
        let key: [u8; 16] = [
            101, 50, 51, 100, 55, 101, 53, 101, 99, 99, 52, 49, 48, 57, 48, 0,
        ];
        let iv: [u8; 16] = [
            159, 83, 25, 66, 156, 217, 148, 45, 151, 246, 253, 223, 7, 117, 64, 0,
        ];
        let plaintext = b"Hello World!".to_vec();
        let ciphertext: Vec<u8> = encrypt_data_sym(&key, &iv, &plaintext);
        let expected_ciphertext = [
            255, 66, 148, 183, 158, 105, 12, 139, 19, 249, 134, 174, 225, 140, 174, 2,
        ];
        assert_eq!(ciphertext, expected_ciphertext);
    }

    #[test]
    fn test_decrypt_data_asym() {
        let ciphertext = vec![
            173, 76, 30, 176, 163, 29, 112, 56, 249, 132, 155, 47, 76, 207, 82, 94, 77, 164, 94,
            248, 222, 148, 247, 166, 231, 39, 83, 124, 188, 126, 104, 29, 86, 135, 171, 135, 219,
            185, 72, 133, 102, 139, 188, 184, 195, 76, 158, 149, 155, 46, 88, 24, 57, 200, 11, 163,
            55, 176, 3, 170, 231, 77, 184, 35, 175, 192, 115, 230, 90, 226, 5, 19, 37, 10, 186,
            227, 84, 194, 55, 192, 29, 106, 21, 146, 89, 161, 150, 231, 101, 109, 219, 6, 86, 96,
            139, 200, 211, 83, 105, 42, 2, 240, 116, 84, 184, 242, 95, 133, 146, 241, 180, 29, 165,
            59, 255, 235, 5, 212, 136, 15, 129, 213, 134, 241, 1, 1, 159, 229, 186, 200, 163, 163,
            80, 225, 156, 139, 98, 64, 254, 210, 25, 205, 21, 126, 64, 247, 81, 58, 241, 29, 215,
            194, 127, 225, 175, 96, 55, 68, 75, 208, 63, 56, 4, 108, 74, 246, 196, 175, 170, 193,
            4, 166, 112, 239, 124, 224, 86, 165, 144, 21, 74, 233, 74, 202, 228, 82, 52, 211, 108,
            122, 58, 61, 55, 44, 175, 122, 165, 17, 71, 121, 37, 31, 118, 64, 45, 131, 241, 90, 99,
            214, 178, 84, 58, 89, 206, 106, 143, 70, 45, 165, 157, 60, 158, 206, 110, 23, 43, 166,
            162, 173, 219, 47, 38, 101, 6, 135, 156, 127, 83, 229, 40, 68, 33, 92, 135, 18, 85,
            166, 49, 178, 30, 31, 238, 77, 252, 152,
        ];
        let private_key: RSAPrivateKey = read_private_key().unwrap();
        let plaintext = decrypt_data_asym(&ciphertext, &private_key);
        let expected_plaintext: Vec<u8> = b"Hello world. This is my plaintext message".to_vec();
        assert_eq!(plaintext, expected_plaintext);
    }

    #[test]
    fn test_decrypt_sym() {
        let key: [u8; 16] = [
            54, 98, 49, 57, 101, 53, 49, 53, 98, 99, 57, 52, 97, 51, 50, 57,
        ];
        let iv: [u8; 16] = [
            52, 56, 49, 48, 54, 56, 97, 97, 56, 98, 48, 52, 53, 97, 51, 101,
        ];
        let ciphertext = [
            169, 199, 157, 209, 50, 249, 171, 226, 126, 140, 92, 56, 244, 119, 75, 35,
        ];
        let plaintext = decrypt_sym(&key, &iv, &ciphertext);
        let expected_plaintext = b"Hello World!".to_vec();
        assert_eq!(plaintext, expected_plaintext);
    }

    #[test]
    fn test_generate_random_hex_16() {
        let hex: [u8; 16] = generate_random_hex_16();
        assert_eq!(hex.len(), 16);
        assert!(hex[15] > 0);
    }

    fn read_public_key() -> Result<RSAPublicKey> {
        let mut file = File::open("./src/test/test.pem.pub")?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
        let public_key = RSAPublicKey::try_from(pem_encoded).expect("failed to parse key");
        Ok(public_key)
    }

    fn read_private_key() -> Result<RSAPrivateKey> {
        let mut file = File::open("./src/test/test.pem")?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
        let private_key = RSAPrivateKey::try_from(pem_encoded).expect("failed to parse key");
        Ok(private_key)
    }
}
