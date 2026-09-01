use lapin::{Channel, Connection, ConnectionProperties};
use url::Url;

pub struct RabbitMQConn {
    uri: Url,
    connection: Option<Connection>,
}

impl RabbitMQConn {
    pub fn new(url: Url) -> Self {
        Self {
            uri: url,
            connection: None,
        }
    }

    pub async fn connect(&mut self) {
        self.connection = Some(
            Connection::connect(self.uri.as_str(), ConnectionProperties::default())
                .await
                .expect("Failed to connect to RabbitMQ"),
        );
    }

    pub async fn create_channel(&self) -> Channel {
        let connection = self.connection.as_ref().expect("Not connected");
        connection
            .create_channel()
            .await
            .expect("Failed to create channel.")
    }

    pub async fn close(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close(200, "OK".into()).await.ok();
        }
    }
}
