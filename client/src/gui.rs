use fltk::{app::*, button::*, input::*, menu::*, text::*, tree::*, window::Window};
use log::debug;
use rsa::RSAPublicKey;
use std::collections::HashMap;

use crate::crypto;
use crate::io;
use crate::prop;
use crate::util;

#[derive(Copy, Clone)]
pub enum Message {
    New,
    Users,
    Quit,
}

// https://github.com/MoAlyousef/fltk-rs/blob/master/src/prelude.rs#L63
pub fn init(window_label: &str) {
    let users: HashMap<String, (String, String)> = io::read_users_file();
    debug!("Loaded users: {:?}", &users);

    let window_width = 800;
    let window_height = 600;
    let padding = 10;
    let users_display_width = 250;
    let armored_out_width = window_width - users_display_width;
    let app = App::default();
    let mut window = Window::default()
        .with_size(window_width, window_height)
        .center_screen()
        .with_label(window_label);

    //Inputs
    let plaintext_in = Input::new(padding, 40 + padding, window_width - 2 * padding, 40, "");
    let armored_in =
        MultilineInput::new(padding, 320 + padding, window_width - 2 * padding, 150, "");

    //Outputs
    let mut armored_out = build_text_display(padding, 100, armored_out_width - 2 * padding, 150);
    let mut users_tree = build_users_tree_select(
        armored_out_width,
        100,
        users_display_width - padding,
        150,
        &users,
    );
    let mut plaintext_out = build_text_display(padding, 490, window_width - 2 * padding, 40);

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
        530 + padding,
        70,
        30,
        "Decrypt",
    );

    //Channels
    let (s, r) = channel::<Message>();

    //Menu
    let mut menu = MenuBar::new(0, 0, window_width, 40, "");
    init_menu(&mut menu, s);

    //Event handling must be done after the drawing is done and the main `window` shown. And must be done in the main thread.
    window.make_resizable(true);
    window.end();
    window.show();

    //Button events
    encrypt_button.set_callback(Box::new(move || {
        let user_name: String = match users_tree.get_selected_items() {
            Some(tree_users) => {
                let user_name: String = tree_users.as_slice()[0].label().unwrap();
                debug!("user {} was selected", &user_name);
                user_name
            }
            None => {
                debug!("No user was selected");
                "".to_string()
            }
        };

        let mut user_id: String = String::new();
        let mut pub_key: String = String::new();
        if let Some((id, key)) = users.get(&user_name) {
            user_id.push_str(id);
            pub_key.push_str(key);
        } else {
            user_id.push_str("self");
        }

        let input: String = plaintext_in.value();
        let result: String = encrypt_text(&input, &pub_key, &user_id);
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

fn build_text_display(x: i32, y: i32, w: i32, h: i32) -> TextDisplay {
    TextDisplay::new(x, y, w, h, TextBuffer::default())
}

fn build_users_tree_select(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    users: &HashMap<String, (String, String)>,
) -> Tree {
    let mut tree = Tree::new(x, y, w, h, "");
    tree.set_select_mode(TreeSelect::Single);
    tree.set_root_label("Slack Users");

    for name in users.keys() {
        tree.add(&name);
    }

    tree
}

fn init_menu(menu: &mut MenuBar, s: Sender<Message>) {
    menu.add(
        "File/New Public Key",
        Shortcut::None,
        MenuFlag::Normal,
        Box::new(move || s.send(Message::New)),
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
}

fn encrypt_text(plaintext: &str, pub_key: &str, user_id: &str) -> String {
    let dir = util::default_dir();

    let public_key: RSAPublicKey = match user_id {
        "self" => io::get_public_key(&dir).unwrap(),
        _user_id => io::parse_public_key(pub_key).unwrap(),
    };

    match crypto::slackrypt(plaintext.as_bytes(), &public_key, user_id) {
        Ok(ascii_message) => ascii_message.into_string(),
        Err(e) => format!("Error trying to build and encrypt message: {}", e),
    }
}

fn decrypt_text(armored_msg: &str) -> String {
    match crypto::unslackrypt(armored_msg) {
        Ok(msg) => msg,
        Err(e) => format!("Error trying to parse and decrypt message: {}", e),
    }
}

fn get_user_pubkeys() {
    let user_pubkeys: Vec<String> = get_pubkeys().unwrap();
    io::update_users_file(user_pubkeys).unwrap();
}

#[tokio::main]
async fn get_pubkeys() -> Result<Vec<String>, reqwest::Error> {
    let base_url: String = prop::get_property("server_base_url", "http://127.0.0.1:8080");
    let endpoint: String = base_url + "/pubkey/users";
    let json_resp: serde_json::Value = reqwest::Client::new()
        .get(&endpoint)
        .send()
        .await?
        .json()
        .await?;

    let resp: String = json_resp.to_string();
    let pubkeys: Vec<String> = serde_json::from_str(&resp).unwrap();
    Ok(pubkeys)
}
