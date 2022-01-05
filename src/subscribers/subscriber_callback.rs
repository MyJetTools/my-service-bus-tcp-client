use async_trait::async_trait;
use my_service_bus_shared::MessageId;

use super::MySbDeliveryPackage;
pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

#[async_trait]
pub trait SubscriberCallback {
    async fn new_events(&self, delivery_package: MySbDeliveryPackage) -> Result<(), String>;
}
