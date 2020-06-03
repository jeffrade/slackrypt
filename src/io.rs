extern crate pem;

use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdout, Result, Write};

use rsa::{RSAPrivateKey, RSAPublicKey};

pub fn get_public_key(dir: &str) -> Result<RSAPublicKey> {
    let file_name = String::from(dir) + "/key.pem.pub";
    let mut file = File::open(file_name)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let pem_encoded = pem::parse(file_content).expect("failed to parse pem file");
    let public_key = RSAPublicKey::try_from(pem_encoded).expect("failed to parse key");
    Ok(public_key)
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

pub fn parse_message_from_file(file_name: &str) -> Result<String> {
    let mut file = File::open(file_name)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

//A psuedo ASCII Armor format https://tools.ietf.org/html/rfc4880#section-6.2
pub fn write_message_to_stdout(
    begin_head: &str,
    end_head: &str,
    ver_head: &str,
    cipher: &str,
    key: &str,
    iv: &[u8],
) -> Result<()> {
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

//A psuedo ASCII Armor format https://tools.ietf.org/html/rfc4880#section-6.2
pub fn write_message_to_file(
    file_name: &str,
    begin_head: &str,
    end_head: &str,
    ver_head: &str,
    cipher: &str,
    key: &str,
    iv: &[u8],
) {
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
