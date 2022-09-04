use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::{MessageToDeliverTcpContract, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;
use rust_extensions::Logger;
use tokio::sync::Mutex;

use super::{
    my_sb_subscribers_data::MySbSubscriber, MessagesReader, MySbSubscribersData, SubscriberCallback,
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
        calback: Arc<dyn SubscriberCallback + Send + Sync + 'static>,
    ) {
        let mut write_access = self.subscribers.lock().await;
        write_access.add(topic_id, queue_id, queue_type, calback);
    }

    pub async fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
        messages: Vec<MessageToDeliverTcpContract>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let callback = {
            let read_access = self.subscribers.lock().await;
            read_access.get_callback(topic_id.as_str(), queue_id.as_str())
        };

        if let Some(callback) = callback {
            let messages_reader = MessagesReader::new(
                topic_id,
                queue_id,
                messages,
                confirmation_id,
                connection,
                logger.clone(),
            );

            tokio::spawn(new_messages_callback(messages_reader, callback, logger));
        }
    }

    pub async fn get_subscribers(&self) -> Vec<MySbSubscriber> {
        let read_access = self.subscribers.lock().await;
        read_access.get_subscribers()
    }
}

async fn new_messages_callback(
    messages_reader: MessagesReader,
    callback: Arc<dyn SubscriberCallback + Sync + Send + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) {
    let topic_id = messages_reader.topic_id.to_string();
    let queue_id = messages_reader.queue_id.to_string();
    let confirmation_id = messages_reader.confirmation_id;

    let result = tokio::spawn(async move {
        callback.new_events(messages_reader).await;
    })
    .await;

    if let Err(err) = result {
        let mut log_context = HashMap::new();
        log_context.insert("ConfirmationId".to_string(), confirmation_id.to_string());

        log_context.insert("TopicId".to_string(), topic_id);
        log_context.insert("QueueId".to_string(), queue_id);

        logger.write_error(
            "MySB Incoming messages".to_string(),
            format!("{}", err),
            Some(log_context),
        )
    }
}
