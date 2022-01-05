use async_trait::async_trait;

use super::MySbDeliveryPackage;

#[async_trait]
pub trait SubscriberCallback {
    async fn new_events(&self, delivery_package: MySbDeliveryPackage) -> Result<(), String>;
}
