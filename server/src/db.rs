use rusqlite::{params, Connection, Result, NO_PARAMS};
use std::vec::Vec;

use crate::util;

#[derive(Debug)]
struct User {
    id: i32,
    user_id: String,
    name: String,
    pubkey: String,
}

fn get_connection() -> Result<Connection> {
    let path: String = util::default_dir() + "/slackrypt.db3";
    let conn = Connection::open(&path)?;
    log::debug!("is autocommit? {}", conn.is_autocommit());
    Ok(conn)
}

pub fn insert_pubkeys(user_info: &[(&str, &str, &str)]) -> Result<()> {
    for (user_id, name, pubkey) in user_info {
        if !has_pubkey(user_id) {
            insert_pubkey(user_id, name, pubkey).unwrap();
        }
    }
    Ok(())
}

fn insert_pubkey(user_id: &str, name: &str, pubkey: &str) -> Result<()> {
    let conn: Connection = get_connection().unwrap();
    conn.execute(
        "INSERT INTO users (user_id, name, pubkey) VALUES (?1, ?2, ?3)",
        params![user_id, name, pubkey],
    )?;
    Ok(())
}

pub fn upsert_pubkey(user_id: &str, name: &str, pubkey: &str) -> Result<()> {
    if has_pubkey(user_id) {
        update_pubkey(user_id, pubkey)
    } else {
        insert_pubkey(user_id, name, pubkey)
    }
}

fn update_pubkey(user_id: &str, pubkey: &str) -> Result<()> {
    let conn: Connection = get_connection().unwrap();
    conn.execute(
        "UPDATE users SET pubkey=?1 WHERE user_id=?2",
        params![pubkey, user_id],
    )?;
    Ok(())
}

fn has_pubkey(user_id: &str) -> bool {
    let pubkeys: Vec<String> = select_pubkey(user_id).unwrap();
    !pubkeys.is_empty()
}

pub fn select_pubkey(user_id: &str) -> Result<Vec<String>> {
    let conn: Connection = get_connection().unwrap();
    let mut stmt = conn.prepare("SELECT pubkey FROM users WHERE user_id = :user_id")?;
    let mut rows = stmt.query_named(&[(":user_id", &user_id)])?;

    let mut pubkeys = Vec::new();
    while let Some(row) = rows.next()? {
        let pubkey: String = row.get(0)?;
        pubkeys.push(pubkey);
    }

    Ok(pubkeys)
}

pub fn get_users_all() -> Result<Vec<String>> {
    let conn: Connection = get_connection().unwrap();
    let mut stmt = conn.prepare("SELECT user_id, name, pubkey FROM users ORDER BY name")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut users = Vec::new();
    while let Some(row) = rows.next()? {
        let user_id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let pubkey: String = row.get(2)?;
        let user_csv = user_id + "," + &name + "," + &pubkey;
        users.push(user_csv);
    }

    Ok(users)
}

pub fn init() -> Result<()> {
    log::info!("Starting SQLite3...");
    let conn: Connection = get_connection().unwrap();

    match conn.execute(
        "CREATE TABLE users (
                  id              INTEGER PRIMARY KEY,
                  user_id         TEXT UNIQUE NOT NULL,
                  name            TEXT NOT NULL,
                  pubkey          TEXT NOT NULL
                  )",
        params![],
    ) {
        Ok(_) => true,
        Err(_) => {
            log::warn!("Ignore since user table might already exist.");
            true
        }
    };

    let users: Vec<String> = get_users_all().unwrap();

    log::debug!("Current users:");
    for user in users {
        log::debug!("{:?}", user);
    }
    Ok(())
}
