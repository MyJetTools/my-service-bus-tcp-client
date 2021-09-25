use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::{TcpContract, TcpContractMessage};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::tcp::SocketConnection;

use super::{subscribe_item::SubscribeItem, MySbDeliveryPackage};

pub struct MySbSubscribers {
    subscribers: Mutex<HashMap<String, HashMap<String, SubscribeItem>>>,
}

impl MySbSubscribers {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(HashMap::new()),
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
        if !write_access.contains_key(topic_id.as_str()) {
            write_access.insert(topic_id.to_string(), HashMap::new());
        }

        let by_topic = write_access.get_mut(topic_id.as_str()).unwrap();

        if by_topic.contains_key(queue_id.as_str()) {
            panic!(
                "Subscriber for topic:{} and queue:{} is already registered",
                topic_id, queue_id
            );
        }

        let item = SubscribeItem::new(topic_id, queue_id.to_string(), queue_type, tx);

        by_topic.insert(queue_id, item);
    }

    pub async fn new_connection(&self, ctx: Arc<SocketConnection>) {
        let read_access = self.subscribers.lock().await;
        for by_topic in read_access.values() {
            for itm in by_topic.values() {
                let tcp_contract = TcpContract::Subscribe {
                    topic_id: itm.topic_id.to_string(),
                    queue_id: itm.queue_id.to_string(),
                    queue_type: itm.queue_type,
                };

                let payload = tcp_contract.serialize(&ctx.attr);

                ctx.send_data_to_socket_and_forget(payload.as_slice()).await;
            }
        }
    }

    pub async fn new_messages(
        &self,
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        messages: Vec<TcpContractMessage>,
    ) {
        let read_access = self.subscribers.lock().await;
        let by_topic = read_access.get(topic_id.as_str());

        if let Some(by_topic) = by_topic {
            if let Some(subscriber) = by_topic.get(queue_id.as_str()) {
                let msg = MySbDeliveryPackage {
                    topic_id,
                    messages: Some(messages),
                    confirmation_id,
                    queue_id,
                };

                let result = subscriber.tx.send(msg);

                if let Err(err) = result {
                    print!("Send Error: {}", err)
                }
            }
        }
    }
}
