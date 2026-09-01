mod config;
mod events;
mod rabbitmq;
mod twitch;

use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use tokio::sync::mpsc;

use crate::events::router::route;
use crate::rabbitmq::rabbitmq::RabbitMQConn;
use crate::twitch::authentication::TwitchAuthenticator;
use crate::twitch::eventsub_types::EventSubType;
use crate::twitch::twitch_event_sub::{TwitchEventSub, WSNotification};

// TODO: graceful shutdown

#[tokio::main]
async fn main() {
    let config = config::deserialize();
    let subscription_types = EventSubType::str_to_event_type(&config.events);

    // authenticate user
    let mut auth =
        TwitchAuthenticator::new(config.app_keys.client_id, config.app_keys.client_secret);
    auth.authenticate(&subscription_types).await.unwrap();

    // connect to rabbitmq
    let mut rabbit_conn = RabbitMQConn::new(url::Url::parse(config.rabbit_mq.rabbit_mq_url.as_str()).unwrap());
    rabbit_conn.connect().await;
    let channel = rabbit_conn.create_channel().await;

    // TODO: declare rabbitMQ exchange/queue

    // Create a new channel with a capacity of at most 32.
    // https://tokio.rs/tokio/tutorial/channels
    let (tx, mut rx) = mpsc::channel::<WSNotification>(32);

    // create socket and subscribe to events
    let broadcaster_id = config.broadcaster_id.clone();
    let mut event_sub = TwitchEventSub::new();
    tokio::spawn(async move {
        event_sub
            .connect(tx, auth, &broadcaster_id, &subscription_types)
            .await;
    });

    while let Some(raw_message) = rx.recv().await {
        // let msg_type = &raw_message.metadata.message_type;
        // println!("GOT = {:?}", msg_type);
        // let sub_type = raw_message.metadata.subscription_type.as_ref().unwrap();
        // println!("{}: {}", sub_type, raw_message.payload);

        let Some(payload) = route(raw_message) else {
            continue;
        };
        let serialized = match serde_json::to_string(&payload) {
            Ok(e) => e,
            Err(_) => continue,
        };

        channel.basic_publish(
            config.rabbit_mq.exchange_name.clone().into(), 
            config.rabbit_mq.routing_key.clone().into(),
            BasicPublishOptions::default(), 
            serialized.as_bytes(),
            BasicProperties::default()
        ).await.unwrap();
    }
}
