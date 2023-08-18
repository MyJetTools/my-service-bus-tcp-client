use std::sync::Arc;

use my_service_bus_abstractions::{
    MySbMessage, MyServiceBusSubscriberClient, MyServiceBusSubscriberClientCallback,
};

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};

use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::Mutex;

use super::MySbSubscribersData;

pub struct MySbSubscribers {
    subscribers: Arc<Mutex<MySbSubscribersData>>,
}

impl MySbSubscribers {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(MySbSubscribersData::new())),
        }
    }

    pub async fn add(
        &self,
        topic_id: &'static str,
        queue_id: String,
        callback: Arc<dyn MyServiceBusSubscriberClientCallback + Send + Sync + 'static>,
    ) {
        let mut write_access = self.subscribers.lock().await;
        write_access.add(topic_id, queue_id, callback);
    }

    pub async fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        connection_id: i32,
        messages: Vec<MySbMessage>,
    ) {
        let callback = {
            let read_access = self.subscribers.lock().await;
            read_access.get_callback(topic_id.as_str(), queue_id.as_str())
        };

        if let Some(callback) = callback {
            callback
                .new_events(messages, confirmation_id, connection_id)
                .await;
        }
    }

    async fn get_subscribers(
        &self,
    ) -> Vec<Arc<dyn MyServiceBusSubscriberClientCallback + Send + Sync + 'static>> {
        let mut result = Vec::new();
        let read_access = self.subscribers.lock().await;

        for subscribers in read_access.subscribers.values() {
            for subscriber in subscribers.values() {
                result.push(subscriber.clone());
            }
        }

        result
    }

    pub async fn new_connection(
        &self,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    ) {
        {
            let mut write_access = self.subscribers.lock().await;
            write_access.connection = Some(connection.clone());
        }

        for subscriber in self.get_subscribers().await {
            let packet = TcpContract::Subscribe {
                topic_id: subscriber.get_topic_id().to_string(),
                queue_id: subscriber.get_queue_id().to_string(),
                queue_type: subscriber.get_queue_type(),
            };

            connection
                .send_bytes(
                    packet
                        .serialize(crate::new_connection_handler::PROTOCOL_VERSION)
                        .as_slice(),
                )
                .await;
        }
    }
    pub async fn disconnect(&self) {
        let mut write_access = self.subscribers.lock().await;
        write_access.connection = None;
    }

    fn send_packet(&self, tcp_contract: TcpContract, connection_id: i32) {
        let subscribers = self.subscribers.clone();

        tokio::spawn(async move {
            let connection = {
                let access = subscribers.lock().await;
                access.connection.clone()
            };

            if let Some(connection) = connection {
                if connection.id == connection_id {
                    connection.send(tcp_contract).await;
                }
            }
        });
    }
}

impl MyServiceBusSubscriberClient for MySbSubscribers {
    fn confirm_delivery(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        connection_id: i32,
        delivered: bool,
    ) {
        let tcp_contract = if delivered {
            TcpContract::NewMessagesConfirmation {
                topic_id: topic_id.to_string(),
                queue_id: queue_id.to_string(),
                confirmation_id,
            }
        } else {
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: topic_id.to_string(),
                queue_id: queue_id.to_string(),
                confirmation_id,
            }
        };

        self.send_packet(tcp_contract, connection_id);
    }

    fn confirm_some_messages_ok(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        connection_id: i32,
        delivered: Vec<my_service_bus_abstractions::queue_with_intervals::QueueIndexRange>,
    ) {
        let tcp_contract = TcpContract::ConfirmSomeMessagesAsOk {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            confirmation_id,
            packet_version: 0,
            delivered,
        };

        self.send_packet(tcp_contract, connection_id);
    }
}
