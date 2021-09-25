use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;

use crate::{subscribers::MySbSubscribers, MySbPublishers};

use super::SocketConnection;

pub async fn connected(
    connection: Arc<SocketConnection>,
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
) {
    let connection_id = connection.id;
    let connected_result =
        tokio::task::spawn(handle_connected(connection, publishers, subscribers)).await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp connected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_connected(
    connection: Arc<SocketConnection>,
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
) {
    publishers.new_connection(connection.clone()).await;
    subscribers.new_connection(connection).await;
}

pub async fn disconnected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublishers>) {
    let connection_id = connection.id;
    let connected_result = tokio::task::spawn(handle_disconnected(connection, publisher)).await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp disconnected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_disconnected(connection: Arc<SocketConnection>, publisher: Arc<MySbPublishers>) {
    publisher.disconnect(connection.id).await;
}

pub async fn new_packet(
    connection: &SocketConnection,
    publisher: &MySbPublishers,
    subscribers: &MySbSubscribers,
    contract: TcpContract,
) {
    match contract {
        TcpContract::PublishResponse { request_id } => {
            publisher.publish_confirmed(connection.id, request_id).await;
        }
        TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            messages,
        } => {
            subscribers
                .new_messages(topic_id, queue_id, confirmation_id, messages)
                .await
        }
        _ => {}
    }
}
