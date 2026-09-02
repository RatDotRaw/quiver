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
        eprintln!("Failed to deserialize chat_message");
        return None;
    };

    println!("channel.chat.message:: {}:{} :: {} :: {}: {}",
        payload.event.broadcaster_user_name,
        payload.event.broadcaster_user_id,
        payload.event.chatter_user_id,
        payload.event.chatter_user_name,
        payload.event.message.text
    );
    return Some(payload);
}
