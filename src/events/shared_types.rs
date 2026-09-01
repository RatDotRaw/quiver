use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub struct Subscription {
    id: String,
    r#type: String,
    version: String,
    cost: u64,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub(crate) text: String,
    pub fragments: Vec<Fragment>,
}

#[derive(Serialize, Deserialize)]
pub struct Fragment {
    #[serde(rename = "type")]
    pub r#type: String,
    pub text: Option<String>,
    pub cheermote: Option<serde_json::Value>,
    pub emote: Option<serde_json::Value>,
    pub gif: Option<serde_json::Value>,
    pub mention: Option<serde_json::Value>,
}