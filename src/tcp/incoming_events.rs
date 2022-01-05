use std::sync::Arc;

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::{ConnectionCallback, ConnectionEvent, SocketConnection};

use crate::{subscribers::MySbSubscribers, MySbPublishers};

pub async fn start(
    mut socket_events_reader: ConnectionCallback<TcpContract, MySbTcpSerializer>,
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    app_name: String,
    client_version: String,
) {
    loop {
        match socket_events_reader.get_next_event().await {
            ConnectionEvent::Connected(connection) => {
                connected(
                    connection,
                    app_name.clone(),
                    client_version.clone(),
                    publishers.clone(),
                    subscribers.clone(),
                )
                .await;
            }
            ConnectionEvent::Disconnected(connection) => {
                disconnected(connection, publishers.clone()).await;
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                new_packet(
                    &connection,
                    publishers.as_ref(),
                    subscribers.as_ref(),
                    payload,
                )
                .await
            }
        }
    }
}

pub async fn connected(
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    app_name: String,
    client_version: String,
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
) {
    let connection_id = connection.id;
    let connected_result = tokio::task::spawn(handle_connected(
        connection,
        app_name,
        client_version,
        publishers,
        subscribers,
    ))
    .await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp connected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_connected(
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    app_name: String,
    client_version: String,
    publishers: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
) {
    publishers.new_connection(connection.clone()).await;
    super::new_connection_handler::send_init(
        connection.as_ref(),
        app_name.as_str(),
        client_version.as_str(),
        publishers.as_ref(),
        subscribers.as_ref(),
    )
    .await;
}

pub async fn disconnected(
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    publishers: Arc<MySbPublishers>,
) {
    let connection_id = connection.id;
    let connected_result = tokio::task::spawn(handle_disconnected(connection, publishers)).await;

    if let Err(err) = connected_result {
        println!(
            "Panic at handeling tcp disconnected event for connection {}. Error: {:?}",
            connection_id, err
        );
    }
}

async fn handle_disconnected(
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    publishers: Arc<MySbPublishers>,
) {
    publishers.disconnect(connection.id).await;
}

pub async fn new_packet(
    connection: &Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    publishers: &MySbPublishers,
    subscribers: &MySbSubscribers,
    contract: TcpContract,
) {
    match contract {
        TcpContract::PublishResponse { request_id } => {
            publishers
                .publish_confirmed(connection.id, request_id)
                .await;
        }
        TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            messages,
        } => {
            subscribers
                .new_messages(
                    topic_id,
                    queue_id,
                    confirmation_id,
                    connection.clone(),
                    messages,
                )
                .await
        }
        _ => {}
    }
}
