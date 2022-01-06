use std::{collections::HashMap, sync::Arc};

use my_logger::MyLogger;
use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
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
    logger: Arc<MyLogger>,
}

impl MessagesReader {
    pub fn new(
        topic_id: String,
        queue_id: String,
        messages: Vec<TcpContractMessage>,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
        logger: Arc<MyLogger>,
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
            logger,
        }
    }

    pub fn handled_ok(&mut self) {
        if let Some(message_id_on_delivery) = self.message_id_on_delivery {
            self.delivered.enqueue(message_id_on_delivery);
            self.message_id_on_delivery = None;
        }
    }

    pub fn handled_fail(&mut self) {
        self.message_id_on_delivery = None;
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
            self.logger.write_log(
                my_logger::LogLevel::Error,
                "Sending delivery confirmation".to_string(),
                "All messages confirmed as fail".to_string(),
                Some(format!(
                    "{}/{}. ConfirmationId: {}",
                    self.topic_id, self.queue_id, self.confirmation_id
                )),
            );

            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
            }
        } else {
            self.logger.write_log(
                my_logger::LogLevel::Error,
                "Sending delivery confirmation".to_string(),
                format!(
                    "{} messages out of {} confirmed as Delivered",
                    self.delivered.len(),
                    self.total_messages_amount
                ),
                Some(format!(
                    "{}/{}. ConfirmationId: {}",
                    self.topic_id, self.queue_id, self.confirmation_id
                )),
            );
            TcpContract::ConfirmSomeMessagesAsOk {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
                delivered: self.delivered.get_snapshot(),
                packet_version: 0,
            }
        };

        let connection = self.connection.clone();

        tokio::spawn(async move {
            connection.send(tcp_packet).await;
        });
    }
}

impl Iterator for &mut MessagesReader {
    type Item = MySbMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.message_id_on_delivery.is_none() {
            panic!("You did not confirm previous message");
        }

        if self.messages.len() == 0 {
            return None;
        }

        let next_message = self.messages.remove(0);
        self.message_id_on_delivery = Some(next_message.id);

        let result = MySbMessage {
            id: next_message.id,
            attempt_no: next_message.attempt_no,
            content: next_message.content,
            headers: next_message.headers,
        };

        Some(result)
    }
}
