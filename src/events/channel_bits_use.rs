use serde::{Deserialize, Serialize};

use crate::{
    events::shared_types::{Message, Subscription},
    twitch::twitch_event_sub::WSNotification,
};

#[derive(Serialize, Deserialize)]
pub struct TwitchBitsUse {
    event: Event,
    subscription: Subscription,
}

// see docs at: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelbitsuse
#[derive(Serialize, Deserialize)]
struct Event {
    user_id: String,
    user_name: String,

    broadcaster_user_id: String,
    broadcaster_user_name: String,

    bits: u32,
    #[serde(rename="type")]
    r#type: String,
    power_up: Option<String>,
    custom_power_up: Option<String>,

    message: Message,
}

pub fn handle_bits_use(notification: WSNotification) -> Option<TwitchBitsUse> {
    let Ok(payload) = serde_json::from_value::<TwitchBitsUse>(notification.payload.clone())
    else {
        eprintln!("Failed to deserialize bits_use");
        return None;
    };

    return Some(payload);
}
