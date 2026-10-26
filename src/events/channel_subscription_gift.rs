use serde::{Deserialize, Serialize};

use crate::{events::shared_types::Subscription, twitch::twitch_event_sub::WSNotification};

#[derive(Serialize, Deserialize)]
pub struct TwitchSubscriptionGift {
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
    total: i32,
    tier: String,
    cumulative_total: i32,
    is_anonymous: bool,
}

pub fn handle_subscription_gift(notification: WSNotification) -> Option<TwitchSubscriptionGift> {

    // Borrowing with & ( serde_json can deserialize from &Value )
    let Ok(payload) = serde_json::from_value::<TwitchSubscriptionGift>(notification.payload.clone())
    else {
        eprintln!("Failed to deserialize subscribe");
        return None;
    };

    println!("channel.subscription.gift:: {}:{} :: {} :: {} gifted {} subs (cumulative: {}) at tier {} (anonymous: {})",
        payload.event.broadcaster_user_name,
        payload.event.broadcaster_user_id,
        payload.event.user_id,
        payload.event.user_name,
        payload.event.total,
        payload.event.cumulative_total,
        payload.event.tier,
        payload.event.is_anonymous
    );
    
    return Some(payload);
}
