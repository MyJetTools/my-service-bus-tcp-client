use std::sync::Arc;

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_tcp_shared::{TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

pub struct MessagesReader {
    pub total_messages: i64,
    pub confirmation_id: i64,
    pub messages: Vec<TcpContractMessage>,
    pub connection: Arc<SocketConnection<TcpContract>>,
    pub topic_id: String,
    pub queue_id: String,
    delivered: QueueWithIntervals,
    on_delivery: Option<MessageId>,
}

impl MessagesReader {
    pub fn new(
        confirmation_id: i64,
        messages: Vec<TcpContractMessage>,
        connection: Arc<SocketConnection<TcpContract>>,
        topic_id: String,
        queue_id: String,
    ) -> Self {
        let total_messages = messages.len() as i64;
        Self {
            confirmation_id,
            messages,
            connection,
            delivered: QueueWithIntervals::new(),
            on_delivery: None,
            topic_id,
            queue_id,
            total_messages,
        }
    }

    pub fn confirm_delivery(&mut self) {
        if let Some(on_delivery) = self.on_delivery {
            self.delivered.enqueue(on_delivery);
            self.on_delivery = None;
        }
    }
}

impl Drop for MessagesReader {
    fn drop(&mut self) {
        let connection = self.connection.clone();

        let tcp_contract = if self.delivered.len() == 0 {
            TcpContract::AllMessagesConfirmedAsFail {
                confirmation_id: self.confirmation_id,
                topic_id: self.topic_id.clone(),
                queue_id: self.queue_id.clone(),
            }
        } else if self.delivered.len() == self.total_messages {
            TcpContract::NewMessagesConfirmation {
                confirmation_id: self.confirmation_id,
                topic_id: self.topic_id.clone(),
                queue_id: self.queue_id.clone(),
            }
        } else {
            TcpContract::ConfirmSomeMessagesAsOk {
                packet_version: 0,
                confirmation_id: self.confirmation_id,
                topic_id: self.topic_id.clone(),
                queue_id: self.queue_id.clone(),
                delivered: self.delivered.get_snapshot(),
            }
        };

        tokio::spawn(async move {
            connection
                .send_bytes(tcp_contract.serialize().as_slice())
                .await
        });
    }
}

impl Iterator for MessagesReader {
    type Item = MySbMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.messages.len() == 0 {
            return None;
        }

        let result = self.messages.remove(0);

        self.on_delivery = Some(result.id);

        let result = MySbMessage {
            id: result.id,
            attempt_no: result.attempt_no,
            content: result.content,
        };

        Some(result)
    }
}
