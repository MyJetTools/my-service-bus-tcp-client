use std::sync::Arc;

use my_service_bus_shared::queue::TopicQueueType;

use super::SubscriberCallback;

pub struct Subscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub callback: Arc<dyn SubscriberCallback + Sync + Send + 'static>,
}

impl Subscriber {
    pub fn new(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        callback: Arc<dyn SubscriberCallback + Sync + Send + 'static>,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            queue_type,
            callback,
        }
    }
}
