use tokio::sync::mpsc::UnboundedSender;

use super::MySbDeliveryConfirmationEvent;

pub struct ConfirmationSender {
    sender: UnboundedSender<MySbDeliveryConfirmationEvent>,
}

impl ConfirmationSender {
    pub fn new(sender: UnboundedSender<MySbDeliveryConfirmationEvent>) -> Self {
        Self { sender }
    }

    pub fn send(&self, event: MySbDeliveryConfirmationEvent) {
        let result = self.sender.send(event);

        if let Err(err) = result {
            println!(
                "Somehow we got an error on sending confirmation event. Err:{}",
                err
            )
        }
    }
}
