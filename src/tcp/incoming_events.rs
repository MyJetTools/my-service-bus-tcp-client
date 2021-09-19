use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::MySbPublisher;

use super::SocketConnection;

pub async fn connected(connection: Arc<SocketConnection>, publisher: &MySbPublisher) {
    publisher.connect(connection).await;
}

pub async fn disconnected(connection: &SocketConnection, publisher: &MySbPublisher) {
    publisher.disconnect(connection.id).await;
}

pub async fn new_packet(
    connection: &SocketConnection,
    publisher: &MySbPublisher,
    contract: TcpContract,
) {
    match contract {
        TcpContract::PublishResponse { request_id } => {
            publisher.publish_confirmed(connection.id, request_id).await;
        }
        _ => {}
    }
}
