use serde::{Deserialize, Serialize};

use crate::{events::shared_types::{Message, Subscription}, twitch::twitch_event_sub::WSNotification};

#[derive(Serialize, Deserialize)]
pub struct TwitchChatMessage {
    event: Event,
    subscription: Subscription,
}

// see docs at: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatmessage
#[derive(Serialize, Deserialize)]
struct Event {
    broadcaster_user_id: String,
    broadcaster_user_name: String,
    chatter_user_id: String,
    chatter_user_name: String,
    message_id: String,
    message: Message,
}

pub fn handle_message(notification: WSNotification) -> Option<TwitchChatMessage> {

    // Borrowing with & ( serde_json can deserialize from &Value )
    let Ok(payload) = serde_json::from_value::<TwitchChatMessage>(notification.payload.clone())
    else {
        eprintln!("hmm");
        return None;
    };

    println!("user:read:chat :: {}: {}", payload.event.chatter_user_name, payload.event.message.text);
    return Some(payload);
}
