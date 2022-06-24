use async_trait::async_trait;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};
use rust_extensions::Logger;
use std::sync::Arc;

use crate::{subscribers::MySbSubscribers, MySbPublishers, MyServiceBusClient};

pub struct IncomingTcpEvents {
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    app_name: String,
    client_version: String,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl IncomingTcpEvents {
    pub fn new(src: &MyServiceBusClient) -> Self {
        Self {
            publishers: src.publishers.clone(),
            subscribers: src.subscribers.clone(),
            app_name: src.app_name.clone(),
            client_version: src.client_version.clone(),
            logger: src.logger.clone(),
        }
    }
    async fn handle_connected(
        &self,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    ) {
        self.publishers.new_connection(connection.clone()).await;
        super::new_connection_handler::send_init(
            connection.as_ref(),
            self.app_name.as_str(),
            self.client_version.as_str(),
            self.publishers.as_ref(),
            self.subscribers.as_ref(),
        )
        .await;
    }

    async fn handle_disconnected(
        &self,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    ) {
        self.publishers.disconnect(connection.id).await;
    }

    pub async fn new_packet(
        &self,
        connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
        contract: TcpContract,
    ) {
        match contract {
            TcpContract::PublishResponse { request_id } => {
                self.publishers
                    .publish_confirmed(connection.id, request_id)
                    .await;
            }
            TcpContract::NewMessages {
                topic_id,
                queue_id,
                confirmation_id,
                messages,
            } => {
                self.subscribers
                    .new_messages(
                        topic_id,
                        queue_id,
                        confirmation_id,
                        connection.clone(),
                        messages,
                        self.logger.clone(),
                    )
                    .await
            }
            _ => {}
        }
    }
}

#[async_trait]
impl SocketEventCallback<TcpContract, MySbTcpSerializer> for IncomingTcpEvents {
    async fn handle(&self, connection_event: ConnectionEvent<TcpContract, MySbTcpSerializer>) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                self.handle_connected(connection).await;
            }
            ConnectionEvent::Disconnected(connection) => {
                self.handle_disconnected(connection).await;
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.new_packet(connection, payload).await,
        }
    }
}
