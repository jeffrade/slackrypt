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

//FIXME INSERTs duplicate record for `user` (instead should UPDATE on `user`)
pub fn insert(user: &str, pubkey: &str) -> Result<()> {
    let conn: Connection = get_connection().unwrap();
    conn.execute(
        "INSERT INTO users (user, pubkey) VALUES (?1, ?2)",
        params![user, pubkey],
    )?;
    Ok(())
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
                  user            TEXT NOT NULL,
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
