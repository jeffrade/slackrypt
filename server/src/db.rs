use log::{info, warn};
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

    // let me = User {
    //     id: 0,
    //     user: "jrade".to_string(),
    //     pubkey: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtcCnEuu9FM08PXCv6gQ7\nIZXhXSKiTYmrkje/aOVqYts9ZV9Gmm1/FFPo5QMtpaIxeLbGKFzgsfM4p1NjClkY\n2izBHhZP8foE9kmoy9rIoPw+jSOocQx5r4Kq8QecWyqhiZvnzvVJVQ3mIwr0zppo\n8D49O1XN23pzcmWrFMyH4c7m7A+xDKwFIVboMKRFGGFAPztHELyi+7bgwuiQcKy+\ncRkfI1+6zLWviWcJttnCWHEAm9qyNxeFBUbmb4exgPogAgpjRXzLCK5TISLnz+NM\nRbtmwhDWFMYnfkv1bskYB8KLeZ9aO6mRgsjX7QyTu71ZEFrDLlxuGl/aa3vYaXx5\nrQIDAQAB\n-----END PUBLIC KEY-----\n".to_string(),
    // };
    // conn.execute(
    //     "INSERT INTO user (user, pubkey) VALUES (?1, ?2)",
    //     params![me.user, me.pubkey],
    // )?;

    let mut stmt = conn.prepare("SELECT id, user, pubkey FROM user")?;
    let user_iter = stmt.query_map(params![], |row| {
        Ok(User {
            id: row.get(0)?,
            user: row.get(1)?,
            pubkey: row.get(2)?,
        })
    })?;

    for user in user_iter {
        info!("Found user {:?}", user.unwrap());
    }
    Ok(())
}
