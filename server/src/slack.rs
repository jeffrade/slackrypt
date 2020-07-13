use log::{debug, info};
use slack::api::rtm::StartResponse;
use slack::api::{Channel, Message, MessageStandard, User};
use slack::{Event, RtmClient};
use std::vec::Vec;

use crate::db;
use crate::util;

//TODO Add a user cache where user_id is the index
struct SlackHandler {
    server_base_url: String,
    direct_msg_prefix: char,
    user_id: String,
    real_name: String,
    reply_pattern: String,
}

impl SlackHandler {
    fn should_reply(&self, event_text: &str) -> bool {
        event_text.starts_with(&self.reply_pattern)
    }

    fn is_direct_msg(&self, channel_id: &str) -> bool {
        channel_id.starts_with(self.direct_msg_prefix)
    }

    fn is_public_key(&self, event_text: &str, channel_id: &str) -> bool {
        self.is_direct_msg(channel_id)
            && event_text.starts_with("-----BEGIN PUBLIC KEY-----")
            && event_text.ends_with("-----END PUBLIC KEY-----")
    }
}

impl slack::EventHandler for SlackHandler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        let mut event_text: String = String::new();
        let mut sender: String = String::new();
        let mut channel_id: String = String::new();
        debug!("\n\n#####################debug event\n{:?}\n\n", event);
        match event {
            Event::Message(message) => match *message {
                Message::Standard(MessageStandard {
                    ref text,
                    ref user,
                    ref channel,
                    ..
                }) => {
                    event_text.push_str(text.as_ref().unwrap());
                    sender.push_str(user.as_ref().unwrap());
                    channel_id.push_str(channel.as_ref().unwrap());
                }
                _ => debug!("Message not decoded, ignore it."),
            },
            _ => debug!("Event not decoded, ignore it."),
        }

        // listen for commands
        if (&event_text == "init" || &event_text == "help") && self.is_direct_msg(&channel_id) {
            let response = "Copy and run this in your terminal:\n`echo \"server_base_url=http://"
                .to_string()
                + &self.server_base_url
                + "\" >> ~/.slackrypt/slackrypt.properties`\n\nAfter that is done, please paste your public key found at `~/.slackrypt/key.pem.pub`";
            let _ = cli.sender().send_message(&channel_id, &response);
        }

        if self.is_public_key(&event_text.trim(), &channel_id) {
            let _ = db::upsert_pubkey(&sender, event_text.trim()).unwrap();
            let response: String =
                format!("Thank you. If you're curious, your Slack id is {}", &sender);
            let _ = cli.sender().send_message(&channel_id, &response);
        }

        if self.should_reply(&event_text) {
            let args: Vec<&str> = event_text.split(' ').collect();
            debug!("args are {:?}", args);
            if args.len() > 1 {
                //add DM commands to take action on here
                if args[1] == "help" {
                    let _ = cli
                        .sender()
                        .send_message(&channel_id, "DM me with the command 'init' to get started.");
                } else {
                    let response: String =
                        format!("I haven't learned how to execute '{}' yet.", args[1]);
                    let _ = cli.sender().send_message(&channel_id, &response);
                }
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        info!("on_close");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        info!("on_connect");
        let channel_name: String = util::get_env_var("SLACK_CHANNEL_NAME", "general");
        let resp: &StartResponse = cli.start_response();
        let users: &Vec<User> = resp.users.as_ref().expect("Could not get users");
        let channels: &Vec<Channel> = resp.channels.as_ref().expect("Could not get channels");

        // find the channel id from the `StartResponse`
        let channel: &Channel = channels
            .iter()
            .find(|c| match c.name {
                None => false,
                Some(ref name) => name == &channel_name,
            })
            .unwrap();
        let channel_id: String = channel.id.as_ref().unwrap().to_string();

        // find all human users to persist initial info
        let mut user_info: Vec<(&str, &str, &str)> = Vec::new();
        for u in users {
            if !u.is_bot.unwrap() && !u.deleted.unwrap() {
                user_info.push((u.id.as_ref().unwrap(), u.name.as_ref().unwrap(), ""));
            }
        }
        db::insert_pubkeys(&user_info).unwrap();

        // find bot user id
        let this_bot_user: &User = users
            .iter()
            .find(|u| match u.profile {
                None => false,
                Some(ref up) => {
                    up.real_name.as_ref().unwrap() == &self.real_name
                        && u.is_bot.unwrap()
                        && !u.deleted.unwrap()
                }
            })
            .unwrap();
        assert_eq!(true, this_bot_user.is_bot.unwrap());
        assert_eq!(false, this_bot_user.deleted.unwrap());
        let this_bot_user_id: String = this_bot_user.id.as_ref().unwrap().to_string();
        self.user_id = String::from(&this_bot_user_id);

        // set the String pattern to look for when responding to @Slackrypt <command>
        self.reply_pattern = "<@".to_string() + &this_bot_user_id + "> ";

        // Send connected message to channel
        let connection_msg: String = format!(
            "I'm up! You can connect to me at {} or DM with 'init'.",
            &self.server_base_url
        );
        let _ = cli.sender().send_message(&channel_id, &connection_msg);
    }
}

pub fn init(server_base_url: &str) {
    info!("Starting Slack RTM client...");
    let api_key: String = util::get_env_var("BOTUSER_AUTH_ACCESS_TOKEN", "");
    let botuser_name: String = util::get_env_var("BOTUSER_REAL_NAME", "Slackrypt");
    let mut slack_handler = SlackHandler {
        server_base_url: server_base_url.to_string(),
        direct_msg_prefix: 'D',
        user_id: "unknown".to_string(),
        real_name: botuser_name,
        reply_pattern: "unknown".to_string(),
    };
    let r = RtmClient::login_and_run(&api_key, &mut slack_handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Err: Could not login and start slack client! {}", err),
    }
}
