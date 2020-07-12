use log::{debug, info, warn};
use rusqlite::{params, Connection, Result, NO_PARAMS};

use crate::util;

#[derive(Debug)]
struct User {
    id: i32,
    user: String,
    pubkey: String,
}

fn get_connection() -> Result<Connection> {
    let path: String = util::default_dir() + "/slackrypt.db3";
    let conn = Connection::open(&path)?;
    debug!("is autocommit? {}", conn.is_autocommit());
    Ok(conn)
}

pub fn upsert_pubkey(user: &str, pubkey: &str) -> Result<()> {
    let pubkeys: Vec<String> = select_pubkey(user).unwrap();
    if pubkeys.is_empty() {
        let conn: Connection = get_connection().unwrap();
        conn.execute(
            "INSERT INTO users (user, pubkey) VALUES (?1, ?2)",
            params![user, pubkey],
        )?;
        Ok(())
    } else {
        update_pubkey(user, pubkey)
    }
}

fn update_pubkey(user: &str, pubkey: &str) -> Result<()> {
    let conn: Connection = get_connection().unwrap();
    conn.execute(
        "UPDATE users SET pubkey=?1 WHERE user=?2",
        params![pubkey, user],
    )?;
    Ok(())
}

pub fn select_pubkey(user: &str) -> Result<Vec<String>> {
    let conn: Connection = get_connection().unwrap();
    let mut stmt = conn.prepare("SELECT pubkey FROM users WHERE user = :user")?;
    let mut rows = stmt.query_named(&[(":user", &user)])?;

    let mut pubkeys = Vec::new();
    while let Some(row) = rows.next()? {
        let pubkey: String = row.get(0)?;
        pubkeys.push(pubkey);
    }

    Ok(pubkeys)
}

pub fn get_users_all() -> Result<Vec<String>> {
    let conn: Connection = get_connection().unwrap();
    let mut stmt = conn.prepare("SELECT user, pubkey FROM users")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut users = Vec::new();
    while let Some(row) = rows.next()? {
        let user: String = row.get(0)?;
        let pubkey: String = row.get(1)?;
        let user_csv = user + "," + &pubkey;
        users.push(user_csv);
    }

    Ok(users)
}

pub fn init() -> Result<()> {
    info!("Starting SQLite3...");
    let conn: Connection = get_connection().unwrap();

    match conn.execute(
        "CREATE TABLE users (
                  id              INTEGER PRIMARY KEY,
                  user            TEXT UNIQUE NOT NULL,
                  pubkey          TEXT NOT NULL
                  )",
        params![],
    ) {
        Ok(_) => true,
        Err(_) => {
            warn!("Ignore since user table might already exist.");
            true
        }
    };

    let users: Vec<String> = get_users_all().unwrap();

    debug!("Current users:");
    for user in users {
        debug!("{:?}", user);
    }
    Ok(())
}
