use std::sync::Arc;

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

pub struct MessagesReader {
    total_messages_amount: i64,
    message_id_on_delivery: Option<MessageId>,
    pub topic_id: String,
    pub queue_id: String,
    messages: Vec<TcpContractMessage>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
}

impl MessagesReader {
    pub fn new(
        topic_id: String,
        queue_id: String,
        messages: Vec<TcpContractMessage>,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    ) -> Self {
        let total_messages_amount = messages.len() as i64;
        Self {
            topic_id,
            queue_id,
            messages,
            confirmation_id,
            connection,
            message_id_on_delivery: None,
            delivered: QueueWithIntervals::new(),
            total_messages_amount,
        }
    }

    pub fn delivered(&mut self) {
        if let Some(message_id_on_delivery) = self.message_id_on_delivery {
            self.delivered.enqueue(message_id_on_delivery);
            self.message_id_on_delivery = None;
        }
    }
}

impl Drop for MessagesReader {
    fn drop(&mut self) {
        let tcp_packet = if self.delivered.len() == self.total_messages_amount {
            TcpContract::NewMessagesConfirmation {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
            }
        } else if self.delivered.len() == 0 {
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
            }
        } else {
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
            }
        };

        let connection = self.connection.clone();

        tokio::spawn(async move {
            connection.send(tcp_packet).await;
        });
    }
}

impl Iterator for MessagesReader {
    type Item = MySbMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.messages.len() == 0 {
            return None;
        }

        let next_message = self.messages.remove(0);
        self.message_id_on_delivery = Some(next_message.id);

        let result = MySbMessage {
            id: next_message.id,
            attempt_no: next_message.attempt_no,
            content: next_message.content,
        };

        Some(result)
    }
}
