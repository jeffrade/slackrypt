use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;

use log::warn;

use crate::util;

pub fn get_properties() -> Result<HashMap<String, String>, Error> {
    let mut props = HashMap::new();
    let path = util::default_dir() + "/" + "slackrypt.properties";
    match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for l in reader.lines() {
                let line: String = l.unwrap();
                let kv: Vec<&str> = line.split('=').collect();
                props.insert(String::from(kv[0]), String::from(kv[1]));
            }
        }
        Err(_e) => {
            warn!("slackrypt.properties file does not exist.");
        }
    }

    Ok(props)
}

pub fn get_property(key: &str, default: &str) -> String {
    match get_properties() {
        Ok(props) => {
            if props.contains_key(key) {
                props.get(key).unwrap().to_string()
            } else {
                String::from(default)
            }
        }
        Err(_) => String::from(default),
    }
}
