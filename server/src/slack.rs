use log::info;
use slack::{Event, RtmClient};

use crate::util;

struct SlackHandler;

impl slack::EventHandler for SlackHandler {
    fn on_event(&mut self, _cli: &RtmClient, event: Event) {
        info!("on_event(event: {:?})", event);
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        info!("on_close");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        info!("on_connect");
        let channel_name: String = util::get_env_var("SLACK_CHANNEL_NAME", "general");
        // find the channel id from the `StartResponse`
        let channel_id = cli
            .start_response()
            .channels
            .as_ref()
            .and_then(|channels| {
                channels.iter().find(|chan| match chan.name {
                    None => false,
                    Some(ref name) => name == &channel_name,
                })
            })
            .and_then(|chan| chan.id.as_ref())
            .expect("channel not found");
        let _ = cli.sender().send_message(&channel_id, "I'm now connected!");
        // Send a message over the real time api websocket
    }
}

pub fn init() {
    info!("Starting Slack RTM client...");
    let api_key: String = util::get_env_var("BOTUSER_AUTH_ACCESS_TOKEN", "");
    let mut slack_handler = SlackHandler;
    let r = RtmClient::login_and_run(&api_key, &mut slack_handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
