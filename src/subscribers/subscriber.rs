use tokio::sync::mpsc::UnboundedReceiver;

use super::{messages_reader::MessagesReader, MySbDeliveryPackage};

pub struct Subscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub receiver: UnboundedReceiver<MySbDeliveryPackage>,
}

impl Subscriber {
    pub fn new(
        topic_id: String,
        queue_id: String,
        receiver: UnboundedReceiver<MySbDeliveryPackage>,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            receiver,
        }
    }

    pub async fn get_next_messages(&mut self) -> MessagesReader {
        loop {
            let recieve_result = self.receiver.recv().await;

            if let Some(package) = recieve_result {
                return MessagesReader::new(
                    package.confirmation_id,
                    package.messages,
                    package.connection.clone(),
                    self.topic_id.to_string(),
                    self.queue_id.to_string(),
                );
            }
        }
    }
}
