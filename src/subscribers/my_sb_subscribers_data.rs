use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue::TopicQueueType;

use super::{subscriber::Subscriber, SubscriberCallback};

pub struct MySbSubscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub queue_type: TopicQueueType,
}

pub struct MySbSubscribersData {
    pub subscribers: HashMap<String, HashMap<String, Subscriber>>,
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
        subscriber_callback: Arc<dyn SubscriberCallback + Sync + Send + 'static>,
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

        let item = Subscriber::new(
            topic_id,
            queue_id.to_string(),
            queue_type,
            subscriber_callback,
        );

        by_topic.insert(queue_id, item);
    }

    pub fn get_callback(
        &self,
        topic_id: &str,
        queue_id: &str,
    ) -> Option<Arc<dyn SubscriberCallback + Sync + Send + 'static>> {
        let by_topic = self.subscribers.get(topic_id)?;

        let subscriber = by_topic.get(queue_id)?;

        return Some(subscriber.callback.clone());
    }

    /*
    pub async fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
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

                let result = subscriber.callback.new_events(msg).await;

                if let Err(err) = result {
                    print!("Send Error: {}", err)
                }
            }
        }
    }

     */

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
