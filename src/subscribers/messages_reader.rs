use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_tcp_shared::TcpContractMessage;

pub struct MessagesReader {
    pub messages: Vec<TcpContractMessage>,

    pub not_delivered: QueueWithIntervals,
}

impl MessagesReader {
    pub fn new(messages: Vec<TcpContractMessage>) -> Self {
        Self {
            messages,
            not_delivered: QueueWithIntervals::new(),
        }
    }
    pub fn not_delivered(&mut self, msg_id: MessageId) {
        self.not_delivered.enqueue(msg_id);
    }
}

impl Drop for MessagesReader {
    fn drop(&mut self) {
        todo!()
    }
}
