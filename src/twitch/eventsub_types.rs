use std::collections::HashMap;

pub enum EventSubType {
    // https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/
    ChannelBitsUse,     // channel.bits.use
    ChannelChatMessage, // channel.chat.message
    ChannelFollow,      // channel.follow
    ChannelRaid,        // channel.raid
    ChannelSubscribe,   // channel.subscribe
}

impl EventSubType {
    /// Resolve scopes required for each given event type
    pub fn resolve_scopes(types: &[EventSubType]) -> Vec<&'static str> {
        let mut result = Vec::new();
        for sub in types {
            for scope in sub.scopes() {
                if !result.contains(scope) {
                    result.push(scope);
                }
            }
        }
        return result;
    }

    /// Convert a subscription type string to the correct enum value.
    pub fn str_to_event_type(list: &[String]) -> Vec<EventSubType> {
        let mut result = Vec::new();
        for e in list {
            match e.as_str() {
                "channel.bits.use" => result.push(EventSubType::ChannelBitsUse),
                "channel.chat.message" => result.push(EventSubType::ChannelChatMessage),
                "channel.follow" => result.push(EventSubType::ChannelFollow),
                "channel.raid" => result.push(EventSubType::ChannelRaid),
                "channel.subscribe" => result.push(EventSubType::ChannelSubscribe),
                _ => eprintln!("EventSubType for '{}' not found.", e),
            }
        }
        return result;
    }

    /// Get the event subscription type name and version.
    pub fn types_info(&self) -> (&'static str, u8) {
        match self {
            EventSubType::ChannelBitsUse => ("channel.bits.use", 1),
            EventSubType::ChannelChatMessage => ("channel.chat.message", 1),
            EventSubType::ChannelFollow => ("channel.follow", 2),
            EventSubType::ChannelRaid => ("channel.raid", 1),
            EventSubType::ChannelSubscribe => ("channel.subscribe", 1),
        }
    }

    /// Resolve enum value to required scopes.
    pub fn scopes(&self) -> &'static [&'static str] {
        match self {
            EventSubType::ChannelBitsUse => &["bits:read"],
            EventSubType::ChannelChatMessage => &["user:read:chat"],
            EventSubType::ChannelFollow => &["moderator:read:followers"],
            EventSubType::ChannelRaid => &[],
            EventSubType::ChannelSubscribe => &["channel:read:subscriptions"],
        }
    }

    pub fn build_condition(
        &self,
        broadcaster_id: String,
        user_id: String,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self {
            EventSubType::ChannelBitsUse => {
                map.insert("broadcaster_user_id".into(), broadcaster_id);
            }
            EventSubType::ChannelChatMessage => {
                map.insert("broadcaster_user_id".into(), broadcaster_id);
                map.insert("user_id".into(), user_id);
            }
            EventSubType::ChannelFollow => {
                map.insert("broadcaster_user_id".into(), broadcaster_id);
                map.insert("moderator_user_id".into(), user_id);
            }
            EventSubType::ChannelRaid => return map,
            EventSubType::ChannelSubscribe => {
                map.insert("broadcaster_user_id".into(), broadcaster_id);
            }
        }
        return map;
    }
}
