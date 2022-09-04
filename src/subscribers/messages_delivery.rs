use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_tcp_shared::{MessageToDeliverTcpContract, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;
use rust_extensions::Logger;

pub struct MySbDeliveredMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}

pub struct MessagesReader {
    total_messages_amount: i64,

    pub topic_id: String,
    pub queue_id: String,
    messages: Option<Vec<MessageToDeliverTcpContract>>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl MessagesReader {
    pub fn new(
        topic_id: String,
        queue_id: String,
        messages: Vec<MessageToDeliverTcpContract>,
        confirmation_id: i64,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let total_messages_amount = messages.len() as i64;
        Self {
            topic_id,
            queue_id,
            messages: Some(messages),
            confirmation_id,
            connection,

            delivered: QueueWithIntervals::new(),
            total_messages_amount,
            logger,
        }
    }

    pub fn handled_ok(&mut self, msg: &MySbDeliveredMessage) {
        self.delivered.enqueue(msg.id);
    }

    pub fn get_messages(&mut self) -> MessagesReaderIterator {
        let mut messages = None;
        std::mem::swap(&mut messages, &mut self.messages);

        if messages.is_none() {
            panic!("Messages can not be iterated for the second time");
        }
        MessagesReaderIterator {
            messages: messages.unwrap(),
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
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.queue_id.to_string());

            self.logger.write_error(
                "Sending delivery confirmation".to_string(),
                "All messages confirmed as fail".to_string(),
                Some(log_context),
            );

            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: self.topic_id.to_string(),
                queue_id: self.queue_id.to_string(),
                confirmation_id: self.confirmation_id,
            }
        } else {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.queue_id.to_string());

            self.logger.write_error(
                "Sending delivery confirmation".to_string(),
                format!(
                    "{} messages out of {} confirmed as Delivered",
                    self.delivered.len(),
                    self.total_messages_amount
                ),
                Some(log_context),
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

pub struct MessagesReaderIterator {
    messages: Vec<MessageToDeliverTcpContract>,
}

impl Iterator for MessagesReaderIterator {
    type Item = MySbDeliveredMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.messages.len() == 0 {
            return None;
        }

        let next_message = self.messages.remove(0);

        let result = MySbDeliveredMessage {
            id: next_message.id,
            attempt_no: next_message.attempt_no,
            content: next_message.content,
            headers: next_message.headers,
        };

        Some(result)
    }
}
