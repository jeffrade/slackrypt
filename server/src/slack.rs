use log::info;
use slack::{Event, RtmClient};

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
        let channel_name = env!("SLACK_CHANNEL_NAME");
        // find the channel id from the `StartResponse`
        let channel_id = cli
            .start_response()
            .channels
            .as_ref()
            .and_then(|channels| {
                channels.iter().find(|chan| match chan.name {
                    None => false,
                    Some(ref name) => name == channel_name,
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
    let api_key = env!("BOTUSER_AUTH_ACCESS_TOKEN").to_string();
    let mut slack_handler = SlackHandler;
    let r = RtmClient::login_and_run(&api_key, &mut slack_handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
