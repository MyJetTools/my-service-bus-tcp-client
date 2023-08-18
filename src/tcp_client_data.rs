use std::sync::{atomic::AtomicBool, Arc};

use my_service_bus_tcp_shared::MySbTcpSerializer;
use my_tcp_sockets::ConnectionEvent;
use rust_extensions::{Logger, StrOrString};

use crate::{publishers::MySbPublishers, subscribers::MySbSubscribers};

pub struct TcpClientData {
    pub app_name: StrOrString<'static>,
    pub app_version: StrOrString<'static>,
    pub client_version: String,
    pub publishers: Arc<MySbPublishers>,
    pub subscribers: Arc<MySbSubscribers>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
    pub has_connection: Arc<AtomicBool>,
}

impl TcpClientData {
    pub async fn new_incoming_data(
        &self,
        connection: Arc<
            my_tcp_sockets::tcp_connection::SocketConnection<
                my_service_bus_tcp_shared::TcpContract,
                MySbTcpSerializer,
            >,
        >,
        contract: my_service_bus_tcp_shared::TcpContract,
    ) {
        match contract {
            my_service_bus_tcp_shared::TcpContract::PublishResponse { request_id } => {
                self.publishers.set_confirmed(request_id).await;
            }
            my_service_bus_tcp_shared::TcpContract::NewMessages {
                topic_id,
                queue_id,
                confirmation_id,
                messages,
            } => {
                self.subscribers
                    .new_messages(topic_id, queue_id, confirmation_id, connection.id, messages)
                    .await
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl my_tcp_sockets::SocketEventCallback<my_service_bus_tcp_shared::TcpContract, MySbTcpSerializer>
    for TcpClientData
{
    async fn handle(
        &self,
        connection_event: ConnectionEvent<
            my_service_bus_tcp_shared::TcpContract,
            MySbTcpSerializer,
        >,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                super::new_connection_handler::send_greeting(
                    &connection,
                    self.app_name.as_str(),
                    self.app_version.as_str(),
                    self.client_version.as_str(),
                )
                .await;

                super::new_connection_handler::send_packet_versions(&connection).await;

                self.publishers.new_connection(connection.clone()).await;
                self.subscribers.new_connection(connection.clone()).await;

                self.has_connection
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
            ConnectionEvent::Disconnected(_) => {
                self.has_connection
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                self.publishers.disconnect().await;
                self.subscribers.disconnect().await;
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.new_incoming_data(connection, payload).await,
        }
    }
}
