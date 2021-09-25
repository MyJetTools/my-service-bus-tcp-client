use my_service_bus_tcp_shared::TcpContractMessage;

use super::messages_reader::MessagesReader;

pub struct MySbDeliveryPackage {
    pub topic_id: String,
    pub queue_id: String,
    pub messages: Option<Vec<TcpContractMessage>>,
    pub confirmation_id: i64,
}

impl MySbDeliveryPackage {
    pub fn read(&mut self) -> MessagesReader {
        let mut messages = None;
        std::mem::swap(&mut messages, &mut self.messages);

        if messages.is_none() {
            panic!("You can read messages only one time");
        }

        MessagesReader::new(messages.unwrap())
    }
}
