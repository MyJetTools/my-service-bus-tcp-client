use std::sync::Arc;

use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::{TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use super::{my_sb_subscribers_data::MySbSubscriber, MySbDeliveryPackage, MySbSubscribersData};

pub struct MySbSubscribers {
    subscribers: Mutex<MySbSubscribersData>,
}

impl MySbSubscribers {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(MySbSubscribersData::new()),
        }
    }

    pub async fn add(
        &self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        tx: UnboundedSender<MySbDeliveryPackage>,
    ) {
        let mut write_access = self.subscribers.lock().await;
        write_access.add(topic_id, queue_id, queue_type, tx);
    }

    pub async fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract>>,
        messages: Vec<TcpContractMessage>,
    ) {
        let read_access = self.subscribers.lock().await;
        read_access.new_messages(topic_id, queue_id, confirmation_id, connection, messages);
    }

    pub async fn get_subscribers(&self) -> Vec<MySbSubscriber> {
        let read_access = self.subscribers.lock().await;
        read_access.get_subscribers()
    }
}
