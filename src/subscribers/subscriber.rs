use std::sync::Arc;

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{
    messages_reader::MessagesReader, my_sb_channel_packages::MySbDeliveryConfirmation,
    ConfirmationSender, MySbDeliveryConfirmationEvent, MySbDeliveryPackage,
};

pub struct Subscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub receiver: UnboundedReceiver<MySbDeliveryPackage>,

    pub confirmations_sender: Arc<ConfirmationSender>,

    confirmation_data: Option<MySbDeliveryConfirmation>,
}

impl Subscriber {
    pub fn new(
        topic_id: String,
        queue_id: String,
        receiver: UnboundedReceiver<MySbDeliveryPackage>,
        confirmations_sender: Arc<ConfirmationSender>,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            receiver,
            confirmation_data: None,
            confirmations_sender,
        }
    }

    fn get_confirmation_data(&mut self) -> Option<MySbDeliveryConfirmation> {
        let mut new_result = None;
        std::mem::swap(&mut new_result, &mut self.confirmation_data);
        new_result
    }

    fn send_confirmation(&mut self) {
        if let Some(confirmation_data) = self.get_confirmation_data() {
            let event = MySbDeliveryConfirmationEvent::Confirmation(confirmation_data);
            self.confirmations_sender.send(event);
        }
    }

    pub async fn get_next_messages(&mut self) -> MessagesReader {
        loop {
            self.send_confirmation();

            let recieve_result = self.receiver.recv().await;

            if let Some(package) = recieve_result {
                self.confirmation_data = Some(MySbDeliveryConfirmation {
                    connection_id: package.connection_id,
                    confirmation_id: package.confirmation_id,
                    not_delivered: None,
                    topic_id: self.topic_id.clone(),
                    queue_id: self.queue_id.clone(),
                });
                return MessagesReader::new(package.confirmation_id, package.messages);
            }
        }
    }

    pub fn confirm_message_is_not_delivered(&mut self, message_id: MessageId) {
        if self.confirmation_data.is_none() {
            panic!("You can not confirm message is not delivered since there are no messages in process");
        }

        if let Some(confirmation_data) = &mut self.confirmation_data {
            if confirmation_data.not_delivered.is_none() {
                confirmation_data.not_delivered = Some(QueueWithIntervals::new())
            }

            if let Some(not_delivered) = &mut confirmation_data.not_delivered {
                not_delivered.enqueue(message_id);
            }
        }
    }
}
