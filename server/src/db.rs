use log::{debug, info, warn};
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct User {
    id: i32,
    user: String,
    pubkey: String,
}

fn get_connection() -> Result<Connection> {
    let path: String = String::from(env!("HOME")) + "/.slackrypt-server" + "/slackrypt.db3";
    let conn = Connection::open(&path)?;
    info!("{}", conn.is_autocommit());
    Ok(conn)
}

pub fn insert(user: &str, pubkey: &str) -> Result<()> {
    let conn: Connection = get_connection().unwrap();
    conn.execute(
        "INSERT INTO user (user, pubkey) VALUES (?1, ?2)",
        params![user, pubkey],
    )?;
    Ok(())
}

pub fn init() -> Result<()> {
    let conn: Connection = get_connection().unwrap();

    match conn.execute(
        "CREATE TABLE user (
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

    let mut stmt = conn.prepare("SELECT id, user, pubkey FROM user")?;
    let user_iter = stmt.query_map(params![], |row| {
        Ok(User {
            id: row.get(0)?,
            user: row.get(1)?,
            pubkey: row.get(2)?,
        })
    })?;

    debug!("Current users:");
    for user in user_iter {
        debug!("{:?}", user.unwrap());
    }
    Ok(())
}
