use serde::{Deserialize, Serialize};

use crate::{events::shared_types::Subscription, twitch::twitch_event_sub::WSNotification};

#[derive(Serialize, Deserialize)]
pub struct TwitchChannelRaid {
    event: Event,
    subscription: Subscription,
}

// see docs at: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelraid
#[derive(Serialize, Deserialize)]
struct Event {
    from_broadcaster_user_id: String,
    from_broadcaster_user_name: String,
    to_broadcaster_user_id: String,
    to_broadcaster_user_name: String,
    viewers: u32,
}

pub fn handle_raid(notification: WSNotification) -> Option<TwitchChannelRaid> {
    // Borrowing with & ( serde_json can deserialize from &Value )
    let Ok(payload) = serde_json::from_value::<TwitchChannelRaid>(notification.payload.clone())
    else {
        eprintln!("Failed to deserialize raid");
        return None;
    };

    println!("channel.raid:: {}:{} :: {} :: {} raided with {} viewers",
        payload.event.to_broadcaster_user_name,
        payload.event.to_broadcaster_user_id,
        payload.event.from_broadcaster_user_id,
        payload.event.from_broadcaster_user_name,
        payload.event.viewers
    );
    return Some(payload);
}
