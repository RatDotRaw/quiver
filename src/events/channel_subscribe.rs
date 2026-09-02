use serde::{Deserialize, Serialize};

use crate::{events::shared_types::Subscription, twitch::twitch_event_sub::WSNotification};

#[derive(Serialize, Deserialize)]
pub struct TwitchSubscribe {
    event: Event,
    subscription: Subscription,
}

// see docs at: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelsubscribe
#[derive(Serialize, Deserialize)]
struct Event {
    user_id: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_name: String,
    tier: String,
    is_gift: bool
}

pub fn handle_subscribe(notification: WSNotification) -> Option<TwitchSubscribe> {

    // Borrowing with & ( serde_json can deserialize from &Value )
    let Ok(payload) = serde_json::from_value::<TwitchSubscribe>(notification.payload.clone())
    else {
        eprintln!("Failed to deserialize subscribe");
        return None;
    };

    println!("channel.subscribe:: {}:{} :: {} :: {} subscribed at tier {} (is_gift: {})",
        payload.event.broadcaster_user_name,
        payload.event.broadcaster_user_id,
        payload.event.user_id,
        payload.event.user_name,
        payload.event.tier,
        payload.event.is_gift
    );
    // todo!("This should get triggered when host raid someone, but no code was written for it.")
    return Some(payload);
}
