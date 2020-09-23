use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Result, Write};

use log::warn;
use rsa::{RSAPrivateKey, RSAPublicKey};

use crate::util;

const USERS_FILE_NAME: &str = "/slackrypt.users";

pub fn get_public_key(dir: &str) -> Result<RSAPublicKey> {
    let file_content: String = get_public_key_string(dir).unwrap();
    parse_public_key(&file_content)
}

pub fn get_public_key_string(dir: &str) -> Result<String> {
    let file_name = String::from(dir) + "/key.pem.pub";
    let mut file = File::open(file_name)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

pub fn parse_public_key(pub_key: &str) -> Result<RSAPublicKey> {
    let pem_encoded = pem::parse(pub_key).expect("failed to parse pem file");
    let public_key = RSAPublicKey::try_from(pem_encoded).expect("failed to parse key");
    Ok(public_key)
}

pub fn get_private_key_default() -> Result<RSAPrivateKey> {
    get_private_key(&util::default_dir())
}

pub fn get_private_key(dir: &str) -> Result<RSAPrivateKey> {
    let file_name = String::from(dir) + "/key.pem";
    let mut file = File::open(file_name)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
    let private_key = RSAPrivateKey::try_from(pem_encoded).expect("failed to parse key");
    Ok(private_key)
}

pub fn load_contents_from_file(file_name: &str) -> Result<String> {
    let mut file = File::open(file_name)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

//A psuedo ASCII Armor format https://tools.ietf.org/html/rfc4880#section-6.2
pub fn build_armor_message(
    begin_head: &str,
    end_head: &str,
    ver_head: &str,
    user_id: &str,
    cipher: &str,
    key: &str,
    iv: &[u8],
) -> String {
    let mut data: String = String::new();
    data.push_str(begin_head);
    data.push_str("\n");
    data.push_str(ver_head);
    data.push_str("\n");
    data.push_str(user_id);
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
    data
}

pub fn update_users_file(users: Vec<String>) -> Result<()> {
    let path = util::default_dir() + USERS_FILE_NAME;
    let mut f = File::create(path)?;

    let mut s = String::new();
    for u in users {
        s.push_str(&u.replace('\n', ""));
        s.push('\n');
    }

    f.write_all(s.as_bytes())
}

pub fn read_users_file() -> HashMap<String, (String, String)> {
    let mut user_pubkey_map = HashMap::new();
    let file_name: String = util::default_dir() + USERS_FILE_NAME;
    match File::open(&file_name) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for l in reader.lines() {
                let line: String = l.unwrap();
                let kv: Vec<&str> = line.splitn(3, ',').collect();
                if kv.len() == 3 && !kv[2].is_empty() {
                    // ignore users with no public key
                    user_pubkey_map.insert(
                        String::from(kv[1]),
                        (String::from(kv[0]), String::from(kv[2])),
                    );
                }
            }
        }
        Err(_e) => {
            warn!("slackrypt.users file does not yet exist.");
        }
    }
    user_pubkey_map
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::crypto;

    #[test]
    fn test_build_armor_message() {
        let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
        let version_header: String = String::from("Version: Slackrypt 0.2");
        let user_id: String = String::from("U1234ABC");
        let ciphertext_b64: String = "qced0TL5q+J+jFw49HdLIw==".to_string();
        let key_b64: String = "Y6gBRPItFubDetqU3fmdWwYQI0Iijr+Zy6IiVESHNT4r+o4DZNdkdDk0YgYXiqwxG07c3wTBpWDX94eriCVEUnJ0WKKmrbPRwI4WpgSb73LwlqlUnNNFS7PnSuCvt7tJ6mJC1QrgO3e2o5j+kiK39ywvwjCQSZsgSIBhJJjuXt5CLKf++r0gpvNYVT9SFGJrslkckdgzszkIMki3QDhSmdDTKGNcaVwDL0w29gIpt1fWQJr7UNxMk6M2hWLHOmXDdNC4Blt6y4ebZxRWH98/mvIAyCFpDlxvVcqILT4tqHJMyNrecNxd2ZzG/p4bScfdEgg2G4d5Lia8ngqmNUhnhw==".to_string();
        let iv: [u8; 16] = [
            101, 50, 51, 100, 55, 101, 53, 101, 99, 99, 52, 49, 48, 57, 48, 0,
        ];
        let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

        let actual = build_armor_message(
            &begin_header,
            &end_header,
            &version_header,
            &user_id,
            &ciphertext_b64,
            &key_b64,
            &iv,
        );

        let expected = "-----BEGIN SLACKRYPT MESSAGE-----\nVersion: Slackrypt 0.2\nU1234ABC\nqced0TL5q+J+jFw49HdLIw==\nY6gBRPItFubDetqU3fmdWwYQI0Iijr+Zy6IiVESHNT4r+o4DZNdkdDk0YgYXiqwxG07c3wTBpWDX94eriCVEUnJ0WKKmrbPRwI4WpgSb73LwlqlUnNNFS7PnSuCvt7tJ6mJC1QrgO3e2o5j+kiK39ywvwjCQSZsgSIBhJJjuXt5CLKf++r0gpvNYVT9SFGJrslkckdgzszkIMki3QDhSmdDTKGNcaVwDL0w29gIpt1fWQJr7UNxMk6M2hWLHOmXDdNC4Blt6y4ebZxRWH98/mvIAyCFpDlxvVcqILT4tqHJMyNrecNxd2ZzG/p4bScfdEgg2G4d5Lia8ngqmNUhnhw==\ne23d7e5ecc41090\u{0}\n-----END SLACKRYPT MESSAGE-----";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_write_and_parse_message_to_file() {
        let public_key: RSAPublicKey = read_public_key().unwrap();

        let begin_header: String = String::from("-----BEGIN SLACKRYPT MESSAGE-----");
        let version_header: String = String::from("Version: Slackrypt 0.2");
        let user_id: String = String::from("U1234ABC");
        let key: [u8; 16] = [
            54, 98, 49, 57, 101, 53, 49, 53, 98, 99, 57, 52, 97, 51, 50, 57,
        ];
        let iv: [u8; 16] = [
            52, 56, 49, 48, 54, 56, 97, 97, 56, 98, 48, 52, 53, 97, 51, 101,
        ];
        let ciphertext = [
            169, 199, 157, 209, 50, 249, 171, 226, 126, 140, 92, 56, 244, 119, 75, 35,
        ];
        let ciphertext_b64: String = util::to_base64_str(&ciphertext);
        let cipher_vec_key: Vec<u8> = crypto::encrypt_data_asym(&key, &public_key);
        let key_b64: String = util::to_base64_str(&cipher_vec_key);
        let end_header: String = String::from("-----END SLACKRYPT MESSAGE-----");

        let file_name: String = util::default_dir() + "/message.test";

        let data = build_armor_message(
            &begin_header,
            &end_header,
            &version_header,
            &user_id,
            &ciphertext_b64,
            &key_b64,
            &iv,
        );
        std::fs::write(&file_name, data).expect("Unable to write encrypted message!");

        //Read encrypted message from the file
        let file_contents: String = load_contents_from_file(&file_name).unwrap();
        let file_lines: Vec<&str> = file_contents.split('\n').collect();
        let version_header_line: &str = file_lines[1];
        assert_eq!(&version_header, &version_header_line);
        let user_id: &str = file_lines[2];
        assert_eq!("U1234ABC", user_id);
        let ciphertext_b64_line: &str = file_lines[3];
        assert_eq!(ciphertext_b64, ciphertext_b64_line);
        let key_b64_line: &str = file_lines[4];
        assert_eq!(key_b64, key_b64_line);
        let iv_line: &str = file_lines[5];
        assert_eq!(&String::from_utf8_lossy(&iv), iv_line);

        std::fs::remove_file(&file_name).expect("message.test not found or permission denied");
    }

    fn read_public_key() -> Result<RSAPublicKey> {
        let mut file = File::open("./src/test/test.pem.pub")?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
        let public_key = RSAPublicKey::try_from(pem_encoded).expect("failed to parse key");
        Ok(public_key)
    }
}
