use serde::{Deserialize, Serialize};

use crate::{events::shared_types::Subscription, twitch::twitch_event_sub::WSNotification};

#[derive(Serialize, Deserialize)]
pub struct TwitchFollow {
    event: Event,
    subscription: Subscription,
}

// see docs at: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelfollow
#[derive(Serialize, Deserialize)]
struct Event {
    user_id: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_name: String,
    followed_at: String,
}

pub fn handle_follow(notification: WSNotification) -> Option<TwitchFollow> {

    // Borrowing with & ( serde_json can deserialize from &Value )
    let Ok(payload) = serde_json::from_value::<TwitchFollow>(notification.payload.clone())
    else {
        eprintln!("hmm");
        return None;
    };

    println!("#@#@#@#@#@#\n{} started following {}\n=@=@=@=@=@=", payload.event.user_name, payload.event.broadcaster_user_name);
    return Some(payload);
}
