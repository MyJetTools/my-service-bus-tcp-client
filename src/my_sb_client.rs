use std::{sync::Arc, time::Duration};

use crate::publishers::PublishError;
use crate::subscribers::{MySbDeliveryPackage, MySbSubscribers};
use crate::MySbPublishers;
use my_service_bus_shared::queue::TopicQueueType;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct MyServiceBusClient {
    host_port: String,
    app_name: String,
    clinet_version: String,
    connect_timeout: Duration,
    ping_timeout: Duration,

    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
}

impl MyServiceBusClient {
    pub fn new(
        host_port: &str,
        app_name: &str,
        clinet_version: &str,
        connect_timeout: Duration,
        ping_timeout: Duration,
    ) -> Self {
        Self {
            host_port: host_port.to_string(),
            app_name: app_name.to_string(),
            clinet_version: clinet_version.to_string(),
            connect_timeout,
            ping_timeout,
            publisher: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
        }
    }

    pub fn start(&self) {
        tokio::task::spawn(crate::tcp::new_connections::start(
            self.host_port.to_string(),
            self.app_name.to_string(),
            self.clinet_version.to_string(),
            self.ping_timeout,
            self.connect_timeout,
            self.publisher.clone(),
            self.subscribers.clone(),
        ));
    }

    pub async fn publish(&self, topic_id: &str, payload: Vec<u8>) -> Result<(), PublishError> {
        self.publisher.publish(topic_id, payload).await?;
        Ok(())
    }

    pub async fn create_topic_if_not_exists(&self, topic_id: String) {
        self.publisher.create_topic_if_not_exists(topic_id).await;
    }

    pub async fn subscribe(
        &self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    ) -> UnboundedReceiver<MySbDeliveryPackage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.subscribers
            .add(topic_id, queue_id, queue_type, tx)
            .await;

        rx
    }
}
