use async_trait::async_trait;

use super::messages_delivery::MessagesReader;

#[async_trait]
pub trait SubscriberCallback {
    async fn new_events(&self, delivery_package: MessagesReader) -> Result<(), String>;
}
