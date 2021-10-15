use std::sync::Arc;

use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::TcpContractMessage;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use super::{
    my_sb_subscribers_data::MySbSubscriber, ConfirmationSender, MySbDeliveryConfirmationEvent,
    MySbDeliveryPackage, MySbSubscribersData,
};

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
        connection_id: i64,
        messages: Vec<TcpContractMessage>,
    ) {
        let read_access = self.subscribers.lock().await;
        read_access.new_messages(topic_id, queue_id, confirmation_id, connection_id, messages);
    }

    pub async fn get_confirmations_sender(&self) -> Arc<ConfirmationSender> {
        let mut write_access = self.subscribers.lock().await;
        write_access.get_confirmations_sender()
    }

    pub async fn get_confirmation_pair(
        &self,
    ) -> (
        Option<Arc<ConfirmationSender>>,
        Option<UnboundedReceiver<MySbDeliveryConfirmationEvent>>,
    ) {
        let mut write_access = self.subscribers.lock().await;
        write_access.get_confirmation_pair()
    }

    pub async fn get_subscribers(&self) -> Vec<MySbSubscriber> {
        let read_access = self.subscribers.lock().await;
        read_access.get_subscribers()
    }
}
