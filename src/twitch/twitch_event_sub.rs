use std::collections::HashMap;

use futures_util::{self, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::twitch::{authentication::TwitchAuthenticator, eventsub_types::EventSubType};

pub struct TwitchEventSub {
    pub session_id: Option<String>,
}

#[derive(serde::Serialize)]
struct EventSubSubscription {
    #[serde(rename = "type")]
    r#type: String,
    version: String,
    condition: HashMap<String, String>, // flexible for different event types
    transport: Transport,
}

#[derive(serde::Serialize)]
struct Transport {
    method: String, // "webhook" | "websocket"
    session_id: String,
}

// see docs at https://dev.twitch.tv/docs/eventsub/handling-websocket-events#notification-message
#[derive(Deserialize)]
pub struct WSNotification {
    pub metadata: WSMetadata,
    pub payload: serde_json::Value,
}

#[derive(Deserialize)]
pub struct WSMetadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
    pub subscription_type: Option<String>,
}

impl TwitchEventSub {
    pub fn new() -> Self {
        Self { session_id: None }
    }

    pub async fn connect(
        &mut self,
        tx: Sender<WSNotification>,
        auth: TwitchAuthenticator,
        broadcaster_id: &str,
        subscription_types: &[EventSubType],
    ) {
        let url: Url = Url::parse("wss://eventsub.wss.twitch.tv/ws").expect("Invalid url");
        let (mut ws_stream, _) = connect_async(url.as_str())
            .await
            .expect("Failed to connect to websocket");
        println!("Websocket client connected!");

        while let Some(raw_msg) = ws_stream.next().await {
            let raw_msg = match raw_msg {
                Ok(val) => val,
                Err(err) => {
                    println!("Received malformed message from server: {:?}", err);
                    continue;
                }
            };

            if raw_msg.is_text() {
                let msg: WSNotification = match serde_json::from_str(raw_msg.to_text().unwrap()) {
                    Ok(msg) => msg,
                    Err(err) => {
                        eprintln!("Failed to parse WS message: {:?}", err);
                        continue;
                    }
                };

                let msg_type = &msg.metadata.message_type;
                let payload = msg.payload.clone();

                // println!("-----------\n{:?}\n-----------", payload);

                if msg_type == "session_welcome" {
                    println!("Received welcome message, subscribing to events...");
                    // set session id
                    self.session_id = Some(payload["session"]["id"].as_str().unwrap().to_string());
                    println!(
                        "Websocket session id received: {}",
                        self.session_id.as_ref().unwrap()
                    );
                    self.subscribe_to_events(
                        &auth,
                        &broadcaster_id.to_string(),
                        subscription_types,
                    )
                    .await;
                    continue;
                } else if msg_type == "session_keepalive" {
                    continue;
                };

                // TODO: check if message is not older than 10 minutes
                // see Guarding against replay attacks https://dev.twitch.tv/docs/eventsub/#guarding-against-replay-attacks

                match tx.send(msg).await {
                    Ok(_) => continue,
                    Err(_) => continue, // TODO: print
                }
            }
        }
    }

    pub async fn subscribe_to_events(
        &self,
        auth: &TwitchAuthenticator,
        broadcaster_id: &String,
        subscription_types: &[EventSubType],
    ) {
        // Build header fields for endpoint
        // https://dev.twitch.tv/docs/api/reference#create-eventsub-subscription
        let mut header_map = HeaderMap::new();
        header_map.insert("Client-Id", HeaderValue::from_str(&auth.client_id).unwrap());
        header_map.insert(
            "Authorization",
            HeaderValue::from_str(&("Bearer ".to_owned() + auth.access_token.as_ref().unwrap()))
                .unwrap(),
        );

        let client = reqwest::Client::new();
        let mut success: u32 = 0;
        let mut failed: u32 = 0;

        for sub_type in subscription_types {
            let (subtype_name, subtype_version) = sub_type.types_info();

            // build request body "condition" field required by endpoint
            let condition_map = sub_type.build_condition(
                broadcaster_id.clone(),
                auth.user_data
                    .as_ref()
                    .expect("User data is empty. Did you run get_user_data?")
                    .id
                    .clone(),
            );

            // build request body
            let body = EventSubSubscription {
                r#type: subtype_name.to_string(),
                version: subtype_version.to_string(),
                condition: condition_map,
                transport: Transport {
                    method: "websocket".to_string(),
                    session_id: self
                        .session_id
                        .clone()
                        .expect("session_id missing. Did you run fn connect()?"),
                },
            };

            // https://dev.twitch.tv/docs/eventsub/manage-subscriptions/
            println!("Subscribing to: {}", subtype_name);

            match client
                .post("https://api.twitch.tv/helix/eventsub/subscriptions")
                .headers(header_map.clone())
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        success += 1;
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        println!(
                            "Couldn't subscribe to {}: {} {}",
                            subtype_name, status, body
                        );
                        failed += 1;
                    }
                }
                Err(err) => {
                    println!("Couldn't subscribe to {}: {}", subtype_name, err);
                    failed += 1;
                }
            }
        }

        println!(
            "Subscription report: {} successful, {} failed",
            success, failed
        );
    }
}
