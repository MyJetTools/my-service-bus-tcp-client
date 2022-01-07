use std::sync::Arc;

use my_logger::MyLogger;
use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::{MessageToDeliverTcpContract, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;
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
        logger: Arc<MyLogger>,
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
    logger: Arc<MyLogger>,
) {
    let topic_id = messages_reader.topic_id.to_string();
    let queue_id = messages_reader.queue_id.to_string();
    let confirmation_id = messages_reader.confirmation_id;

    let result = tokio::spawn(async move {
        callback.new_events(messages_reader).await;
    })
    .await;

    if let Err(err) = result {
        logger.write_log(
            my_logger::LogLevel::FatalError,
            "MySB Incoming messages".to_string(),
            format!("{}", err),
            Some(format!(
                "{}/{} with confirmation id: {}",
                topic_id, queue_id, confirmation_id
            )),
        )
    }
}
