use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::MySbPublisher;

use super::SocketConnection;

pub async fn connected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublisher>) {
    let connection_id = connection.id;
    let connected_result = tokio::task::spawn(handle_connected(connection, publisher)).await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp connected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_connected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublisher>) {
    publisher.connect(connection).await;
}

pub async fn disconnected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublisher>) {
    let connection_id = connection.id;
    let connected_result = tokio::task::spawn(handle_disconnected(connection, publisher)).await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp disconnected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_disconnected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublisher>) {
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
