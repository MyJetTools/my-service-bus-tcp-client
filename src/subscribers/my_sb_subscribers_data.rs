use std::{collections::HashMap, sync::Arc};

use my_service_bus_abstractions::MyServiceBusSubscriberClientCallback;
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct MySbSubscribersData {
    pub subscribers: HashMap<
        &'static str,
        HashMap<String, Arc<dyn MyServiceBusSubscriberClientCallback + Send + Sync + 'static>>,
    >,
    pub connection: Option<Arc<SocketConnection<TcpContract, MySbTcpSerializer>>>,
}

impl MySbSubscribersData {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            connection: None,
        }
    }

    pub fn add(
        &mut self,
        topic_id: &'static str,
        queue_id: String,
        subscriber_callback: Arc<dyn MyServiceBusSubscriberClientCallback + Sync + Send + 'static>,
    ) {
        if !self.subscribers.contains_key(topic_id) {
            self.subscribers.insert(topic_id, HashMap::new());
        }

        let by_topic = self.subscribers.get_mut(topic_id).unwrap();

        if by_topic.contains_key(queue_id.as_str()) {
            panic!(
                "Subscriber for topic:{} and queue:{} is already registered",
                topic_id, queue_id
            );
        }

        by_topic.insert(queue_id, subscriber_callback);
    }

    pub fn get_callback(
        &self,
        topic_id: &str,
        queue_id: &str,
    ) -> Option<Arc<dyn MyServiceBusSubscriberClientCallback + Sync + Send + 'static>> {
        let by_topic = self.subscribers.get(topic_id)?;

        let subscriber = by_topic.get(queue_id)?;

        return Some(subscriber.clone());
    }
}
