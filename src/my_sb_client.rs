use std::{sync::Arc, time::Duration};

use crate::logger::MySbLoggerReader;
use crate::publishers::PublishError;
use crate::subscribers::{MySbSubscribers, Subscriber};
use crate::{MySbLogger, MySbPublishers};
use my_service_bus_shared::queue::TopicQueueType;

pub struct MyServiceBusClient {
    host_port: String,
    app_name: String,
    clinet_version: String,
    connect_timeout: Duration,
    ping_timeout: Duration,

    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    logger: Option<MySbLogger>,
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
            logger: Some(MySbLogger::new()),
        }
    }

    pub async fn start(&mut self) {
        let mut logger = None;
        std::mem::swap(&mut logger, &mut self.logger);

        if logger.is_none() {
            panic!("Client is already started");
        }

        let (confirmation_tx, confirmation_rx) = self.subscribers.get_confirmation_pair().await;

        tokio::task::spawn(crate::tcp::new_connections::start(
            Arc::new(logger.unwrap()),
            self.host_port.to_string(),
            self.app_name.to_string(),
            self.clinet_version.to_string(),
            self.ping_timeout,
            self.connect_timeout,
            self.publisher.clone(),
            self.subscribers.clone(),
            confirmation_tx,
        ));

        if let Some(confirmations_receiver) = confirmation_rx {
            tokio::task::spawn(crate::subscribers::my_sb_subscribers_loop::start(
                confirmations_receiver,
            ));
        }
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
    ) -> Subscriber {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.subscribers
            .add(topic_id.clone(), queue_id.clone(), queue_type, tx)
            .await;

        let confirmaitions_sender = self.subscribers.get_confirmations_sender().await;
        Subscriber::new(topic_id, queue_id, rx, confirmaitions_sender)
    }

    pub fn get_logger_reader(&mut self) -> MySbLoggerReader {
        if self.logger.is_none() {
            panic!("Logger reader can not be extracted because client is already started");
        }

        self.logger.as_mut().unwrap().get_reader()
    }
}
