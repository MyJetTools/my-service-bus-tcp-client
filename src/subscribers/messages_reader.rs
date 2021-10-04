use std::ptr::NonNull;

use my_service_bus_shared::MessageId;
use my_service_bus_tcp_shared::TcpContractMessage;

pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

pub struct MessagesReader {
    pub confirmation_id: i64,
    pub messages: Vec<TcpContractMessage>,
}

impl MessagesReader {
    pub fn new(confirmation_id: i64, messages: Vec<TcpContractMessage>) -> Self {
        Self {
            confirmation_id,
            messages,
        }
    }

    pub fn iterate(&mut self) {}
}

impl Iterator for MessagesReader {
    type Item = MySbMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.messages.len() == 0 {
            return None;
        }

        let result = self.messages.remove(0);

        let result = MySbMessage {
            id: result.id,
            attempt_no: result.attempt_no,
            content: result.content,
        };

        Some(result)
    }
}
