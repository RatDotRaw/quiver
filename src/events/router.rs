use serde::Serialize;

use crate::{
    events::{
        channel_bits_use::{TwitchBitsUse, handle_bits_use}, channel_chat_message::{TwitchChatMessage, handle_message}, channel_follow::{TwitchFollow, handle_follow}, channel_raid::{TwitchChannelRaid, handle_raid}, channel_subscribe::{TwitchSubscribe, handle_subscribe},
    }, twitch::twitch_event_sub::WSNotification,
};

#[derive(Serialize)]
pub enum TwitchEvent { 
    BitsUse(TwitchBitsUse), 
    ChatMessage(TwitchChatMessage), 
    Follow(TwitchFollow), 
    Raid(TwitchChannelRaid), 
    Subscribe(TwitchSubscribe)
}

pub fn route(notification: WSNotification) -> Option<TwitchEvent> {
    let event_type = notification
        .metadata
        .subscription_type
        .as_ref()
        .expect("subscription_type is empty");

    
    let result: Option<TwitchEvent> = match event_type.as_str() {
        "channel.bits.use" => handle_bits_use(notification).map(TwitchEvent::BitsUse),
        "channel.chat.message" => handle_message(notification).map(TwitchEvent::ChatMessage),
        "channel.follow" => handle_follow(notification).map(TwitchEvent::Follow),
        "channel.raid" => handle_raid(notification).map(TwitchEvent::Raid),
        "channel.subscribe" => handle_subscribe(notification).map(TwitchEvent::Subscribe),
        _ => { 
            println!("Router encountered unkown condition: {}", event_type);
            None
        }
    };

    return result
}
