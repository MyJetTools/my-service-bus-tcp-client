use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::{TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::mpsc::UnboundedSender;

use super::{subscribe_item::SubscribeItem, MySbDeliveryPackage};

pub struct MySbSubscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub queue_type: TopicQueueType,
}

pub struct MySbSubscribersData {
    pub subscribers: HashMap<String, HashMap<String, SubscribeItem>>,
}

impl MySbSubscribersData {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        tx: UnboundedSender<MySbDeliveryPackage>,
    ) {
        if !self.subscribers.contains_key(topic_id.as_str()) {
            self.subscribers
                .insert(topic_id.to_string(), HashMap::new());
        }

        let by_topic = self.subscribers.get_mut(topic_id.as_str()).unwrap();

        if by_topic.contains_key(queue_id.as_str()) {
            panic!(
                "Subscriber for topic:{} and queue:{} is already registered",
                topic_id, queue_id
            );
        }

        let item = SubscribeItem::new(topic_id, queue_id.to_string(), queue_type, tx);

        by_topic.insert(queue_id, item);
    }

    pub fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract>>,
        messages: Vec<TcpContractMessage>,
    ) {
        let by_topic = self.subscribers.get(topic_id.as_str());

        if let Some(by_topic) = by_topic {
            if let Some(subscriber) = by_topic.get(queue_id.as_str()) {
                let msg = MySbDeliveryPackage {
                    messages,
                    confirmation_id,
                    connection,
                };

                let result = subscriber.tx.send(msg);

                if let Err(err) = result {
                    print!("Send Error: {}", err)
                }
            }
        }
    }

    pub fn get_subscribers(&self) -> Vec<MySbSubscriber> {
        let mut result = Vec::new();

        for queues in self.subscribers.values() {
            for (_, itm) in queues {
                result.push(MySbSubscriber {
                    topic_id: itm.topic_id.to_string(),
                    queue_id: itm.queue_id.to_string(),
                    queue_type: itm.queue_type,
                });
            }
        }

        result
    }
}
