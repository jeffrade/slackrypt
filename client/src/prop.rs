use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;

use log::warn;

use crate::util;

const PROP_FILE_NAME: &str = "/slackrypt.properties";

pub fn get_properties() -> Result<HashMap<String, String>, Error> {
    let mut props = HashMap::new();
    let path = util::default_dir() + PROP_FILE_NAME;
    match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for l in reader.lines() {
                let line: String = l.unwrap();
                let kv: Vec<&str> = line.splitn(2, '=').collect();
                if kv.len() == 2 {
                    props.insert(String::from(kv[0]), String::from(kv[1]));
                }
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

pub fn upsert_property(key: &str, new_value: &str) -> Result<(), Error> {
    let mut curr_props: HashMap<String, String> = get_properties().unwrap();
    if curr_props.contains_key(key) {
        if let Some(curr_value) = curr_props.get_mut(key) {
            *curr_value = new_value.to_string();
        }
    } else {
        curr_props.insert(key.to_string(), new_value.to_string());
    }
    write_properties(curr_props)
}

fn write_properties(props: HashMap<String, String>) -> Result<(), Error> {
    let path = util::default_dir() + PROP_FILE_NAME;
    let mut f = File::create(path)?;
    let mut s = String::new();

    for (k, v) in props {
        s.push_str(&k);
        s.push('=');
        s.push_str(&v);
        s.push('\n');
    }

    f.write_all(s.as_bytes())
}
