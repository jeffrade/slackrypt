use fltk::{app::*, button::*, input::*, menu::*, text::*, window::Window};
use log::debug;
use rsa::RSAPublicKey;

use crate::crypto;
use crate::io;
use crate::prop;
use crate::util;

#[derive(Copy, Clone)]
pub enum Message {
    New,
    Upload,
    Users,
    Quit,
}

// https://github.com/MoAlyousef/fltk-rs/blob/master/src/prelude.rs#L63
pub fn init(window_label: &str) {
    let window_width = 800;
    let window_height = 600;
    let padding = 10;
    let app = App::default();
    let mut window = Window::default()
        .with_size(window_width, window_height)
        .center_screen()
        .with_label(window_label);

    //Inputs
    let plaintext_in = Input::new(padding, 40 + padding, window_width - 2 * padding, 40, "");
    let armored_in = Input::new(padding, 340 + padding, window_width - 2 * padding, 150, "");

    //Outputs
    let mut armored_out = TextDisplay::new(
        padding,
        100,
        window_width - 2 * padding,
        150,
        TextBuffer::default(),
    );
    let mut plaintext_out = TextDisplay::new(
        padding,
        510,
        window_width - 2 * padding,
        40,
        TextBuffer::default(),
    );

    //Buttons
    let button_width = 70;
    let mut encrypt_button = Button::new(
        window_width / 2 - button_width / 2,
        250 + padding,
        70,
        30,
        "Encrypt",
    );
    let mut decrypt_button = Button::new(
        window_width / 2 - button_width / 2,
        550 + padding,
        70,
        30,
        "Decrypt",
    );

    //Channels
    let (s, r) = channel::<Message>();

    //Menu
    let mut menu = MenuBar::new(0, 0, window_width, 40, "");

    menu.add(
        "File/New Public Key",
        Shortcut::None,
        MenuFlag::Normal,
        Box::new(move || s.send(Message::New)),
    );

    menu.add(
        "File/Upload Public Key",
        Shortcut::None,
        MenuFlag::Normal,
        Box::new(move || s.send(Message::Upload)),
    );

    menu.add(
        "File/Download Public Keys",
        Shortcut::None,
        MenuFlag::Normal,
        Box::new(move || s.send(Message::Users)),
    );

    menu.add(
        "File/Quit",
        Shortcut::None,
        MenuFlag::Normal,
        Box::new(move || s.send(Message::Quit)),
    );

    //Event handling must be done after the drawing is done and the main `window` shown. And must be done in the main thread.
    window.make_resizable(true);
    window.end();
    window.show();

    //Button events
    encrypt_button.set_callback(Box::new(move || {
        let input: String = plaintext_in.value();
        let result: String = encrypt_text(&input);
        armored_out.set_buffer(TextBuffer::default());
        armored_out.buffer().append(&result);
    }));

    decrypt_button.set_callback(Box::new(move || {
        let input: String = armored_in.value();
        let result: String = decrypt_text(&input);
        plaintext_out.set_buffer(TextBuffer::default());
        plaintext_out.buffer().append(&result);
    }));

    window.set_callback(Box::new(move || {
        if event() == Event::Close {
            s.send(Message::Quit);
        }
    }));

    while app.wait().unwrap() {
        use Message::*;

        if let Some(msg) = r.recv() {
            match msg {
                New => {
                    println!("New not implemented!");
                }
                Upload => {
                    upload_pubkey();
                }
                Users => {
                    get_user_pubkeys();
                }
                Quit => {
                    app.quit();
                }
            }
        }
    }

    //Run
    app.run().unwrap();
}

fn encrypt_text(plaintext: &str) -> String {
    let dir = util::default_dir();
    let public_key: RSAPublicKey = io::get_public_key(&dir).unwrap();
    crypto::slackrypt(plaintext.as_bytes(), &public_key)
}

fn decrypt_text(armored_msg: &str) -> String {
    crypto::unslackrypt(armored_msg)
}

fn upload_pubkey() {
    let dir = util::default_dir();
    let pubkey: String = io::get_public_key_string(&dir).unwrap();
    upload("tester", &pubkey).unwrap(); //FIXME get user (id, handle, username?) from slackrypt-server
    println!("{}", &pubkey);
}

fn get_user_pubkeys() {
    let user_pubkeys: Vec<String> = get_pubkeys().unwrap();
    io::update_users_file(user_pubkeys).unwrap();
}

#[tokio::main]
async fn get_pubkeys() -> Result<Vec<String>, reqwest::Error> {
    let host: String = prop::get_property("host", "127.0.0.1");
    let port: String = prop::get_property("port", "8080");
    let url: String = String::from("http://") + &host + ":" + &port + "/pubkey/users";
    let json_resp: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    debug!("{:#?}", json_resp);
    let resp: String = json_resp.to_string();
    let pubkeys: Vec<String> = serde_json::from_str(&resp).unwrap();
    Ok(pubkeys)
}

#[tokio::main]
async fn upload(user: &str, pubkey: &str) -> Result<(), reqwest::Error> {
    let host: String = prop::get_property("host", "127.0.0.1");
    let port: String = prop::get_property("port", "8080");
    let url: String = String::from("http://") + &host + ":" + &port + "/pubkey/upload";
    let json_resp: serde_json::Value = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "user": user,
            "pubkey": pubkey
        }))
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", json_resp);
    Ok(())
}
