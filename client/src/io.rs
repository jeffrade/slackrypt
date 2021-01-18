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

    #[test]
    fn test_parse_public_key() {
        let mut file = File::open("./src/test/test.pem.pub").unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let public_key: Result<RSAPublicKey> = parse_public_key(&file_content);
        assert_eq!(public_key.is_ok(), true);
    }
}
