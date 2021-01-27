use rocket_contrib::json::JsonValue;

use crate::db;
use crate::util;

pub fn start_server() {
    log::info!("Starting HTTP service...");
    rocket::ignite()
        .mount("/", routes![init_sh, pubkey_users])
        .launch();
}

/// curl -H "Content-Type: text/plain" http://127.0.0.1:8000/init.sh
#[get("/init.sh")]
fn init_sh() -> String {
    log::debug!("init_sh() entering...");
    let server_base_url: String = util::get_env_var("SLACKRYPT_BASE_URL", "127.0.0.1:8000");
    util::get_init_sh_cmd(format!("https://{}", &server_base_url).as_str())
}

/// curl -H "Content-Type: application/json" http://127.0.0.1:8000/pubkey/users
#[get("/pubkey/users")]
fn pubkey_users() -> Option<JsonValue> {
    log::debug!("pubkey_users() entering...");
    let users: Vec<String> = db::get_users_all().unwrap();
    if users.is_empty() {
        None
    } else {
        Some(json!(users))
    }
}
