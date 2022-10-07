use std::sync::Arc;

use my_service_bus_abstractions::{MessageToPublish, MyServiceBusPublisherClient, PublishError};
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::Mutex;

use super::{MySbPublisherData, PublishProcessByConnection};

pub struct MySbPublishers {
    data: Mutex<MySbPublisherData>,
}

impl MySbPublishers {
    pub fn new() -> Self {
        let data = MySbPublisherData::new();
        Self {
            data: Mutex::new(data),
        }
    }

    pub async fn set_confirmed(&self, request_id: i64) {
        let mut write_access = self.data.lock().await;
        write_access.confirm(request_id).await;
    }

    pub async fn new_connection(&self, ctx: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>) {
        let mut write_access = self.data.lock().await;

        if let Some(current_connection) = &write_access.connection {
            panic!("We are trying to insert new connection with Id {}, but we have connection {} not disconnected", 
            ctx.id,  current_connection.socket.id,);
        }

        write_access.connection = Some(PublishProcessByConnection::new(ctx));
    }

    pub async fn disconnect(&self) {
        let mut write_access = self.data.lock().await;
        write_access.disconnect();
    }

    pub async fn create_topic_if_not_exists(&self, topic_id: String) {
        let mut write_access = self.data.lock().await;
        write_access.topics_to_create.insert(topic_id, 0);
    }

    pub async fn get_topics_to_create(&self) -> Vec<String> {
        let mut result = Vec::new();
        let write_access = self.data.lock().await;

        for topic_id in write_access.topics_to_create.keys() {
            result.push(topic_id.to_string());
        }

        result
    }
}

#[async_trait::async_trait]
impl MyServiceBusPublisherClient for MySbPublishers {
    async fn publish_message(
        &self,
        topic_id: &str,
        message: MessageToPublish,
    ) -> Result<(), PublishError> {
        return self.publish_messages(topic_id, vec![message]).await;
    }

    async fn publish_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
    ) -> Result<(), PublishError> {
        let awaiter = {
            let mut write_access = self.data.lock().await;

            let (request_id, tcp_contract) = write_access
                .compile_publish_payload(topic_id, messages)
                .await?;

            write_access
                .publish_to_socket(&tcp_contract, request_id)
                .await
        };

        awaiter.get_result().await
    }

    async fn publish_message_with_retries(
        &self,
        topic_id: &str,
        message: MessageToPublish,
        retries_amount: usize,
        retry_delay: std::time::Duration,
    ) -> Result<(), PublishError> {
        self.publish_messages_with_retries(topic_id, vec![message], retries_amount, retry_delay)
            .await
    }

    async fn publish_messages_with_retries(
        &self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
        retries_amount: usize,
        retry_delay: std::time::Duration,
    ) -> Result<(), PublishError> {
        let mut to_send = None;
        let mut messages = Some(messages);
        let mut attempt_no = 0;
        loop {
            if attempt_no > 0 {
                tokio::time::sleep(retry_delay).await;
            }

            attempt_no += 1;

            let awaiter = {
                let mut write_access = self.data.lock().await;

                if to_send.is_none() {
                    let result = write_access
                        .compile_publish_payload(topic_id, messages.take().unwrap())
                        .await;

                    match result {
                        Ok(result) => to_send = Some(result),
                        Err(err) => {
                            if attempt_no >= retries_amount {
                                return Err(err);
                            }
                            continue;
                        }
                    }
                }
                let (request_id, tcp_contract) = to_send.as_ref().unwrap();

                write_access
                    .publish_to_socket(tcp_contract, *request_id)
                    .await
            };

            match awaiter.get_result().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= retries_amount {
                        return Err(err);
                    }
                    continue;
                }
            }
        }
    }
}
