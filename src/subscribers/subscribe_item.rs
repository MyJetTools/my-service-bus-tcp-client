use my_service_bus_shared::queue::TopicQueueType;
use tokio::sync::mpsc::UnboundedSender;

use super::MySbDeliveryPackage;

pub struct SubscribeItem {
    pub topic_id: String,
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub tx: UnboundedSender<MySbDeliveryPackage>,
}

impl SubscribeItem {
    pub fn new(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        tx: UnboundedSender<MySbDeliveryPackage>,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            queue_type,
            tx,
        }
    }
}
