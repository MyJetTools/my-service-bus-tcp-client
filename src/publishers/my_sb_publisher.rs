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

    async fn wait_until_connection_is_restored(&self) {
        loop {
            let has_connection = {
                let read_access = self.data.lock().await;
                read_access.connection.is_some()
            };

            if has_connection {
                return;
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

#[async_trait::async_trait]
impl MyServiceBusPublisherClient for MySbPublishers {
    async fn publish_message(
        &self,
        topic_id: &str,
        message: MessageToPublish,
        do_retry: bool,
    ) -> Result<(), PublishError> {
        return self
            .publish_messages(topic_id, vec![message], do_retry)
            .await;
    }

    async fn publish_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
        do_retries: bool,
    ) -> Result<(), PublishError> {
        let mut to_send = None;
        let mut messages = Some(messages);

        loop {
            let result = {
                let mut write_access = self.data.lock().await;

                let result = if to_send.is_none() {
                    let result = write_access
                        .compile_publish_payload(topic_id, messages.take().unwrap())
                        .await;

                    match result {
                        Ok(result) => {
                            to_send = Some(result);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    Ok(())
                };

                match result {
                    Ok(_) => {
                        let (request_id, tcp_contract) = to_send.as_ref().unwrap();

                        let awaiter = write_access
                            .publish_to_socket(tcp_contract, *request_id)
                            .await;

                        awaiter.get_result().await
                    }
                    Err(err) => Err(err),
                }
            };

            if !do_retries {
                return result;
            }

            if result.is_ok() {
                return Ok(());
            }

            match result.unwrap_err() {
                PublishError::NoConnectionToPublish => {
                    self.wait_until_connection_is_restored().await;
                }
                PublishError::Disconnected => {
                    self.wait_until_connection_is_restored().await;
                }
                PublishError::Other(other) => {
                    return Err(PublishError::Other(other));
                }
                PublishError::SerializationError(err) => {
                    return Err(PublishError::SerializationError(err));
                }
            }
        }
    }
}
