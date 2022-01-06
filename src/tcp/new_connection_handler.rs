use std::collections::HashMap;

use my_service_bus_tcp_shared::{ConnectionAttributes, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

use crate::{subscribers::MySbSubscribers, MySbPublishers};

pub async fn send_init(
    connection: &SocketConnection<TcpContract, MySbTcpSerializer>,
    app_name: &str,
    client_version: &str,
    publisher: &MySbPublishers,
    subscribers: &MySbSubscribers,
) {
    send_packet_versions(connection).await;
    send_greeting(connection, app_name, client_version).await;
    let topics = publisher.get_topics_to_create().await;
    create_topics_if_not_exists(connection, topics).await;
    subscribe_to_queues(connection, subscribers).await;
}

async fn send_greeting(
    socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>,
    app_name: &str,
    client_version: &str,
) {
    let greeting = TcpContract::Greeting {
        name: format!("{};{}", app_name, client_version),
        protocol_version: PROTOCOL_VERSION,
    };

    let payload = greeting.serialize(PROTOCOL_VERSION);
    socket_ctx.send_bytes(payload.as_slice()).await;
}

async fn send_packet_versions(socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    let packet_versions = TcpContract::PacketVersions { packet_versions };
    let payload = packet_versions.serialize(PROTOCOL_VERSION);

    socket_ctx.send_bytes(payload.as_slice()).await;
}

async fn create_topics_if_not_exists(
    socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>,
    topics: Vec<String>,
) {
    for topic_id in topics {
        let packet = TcpContract::CreateTopicIfNotExists { topic_id };

        socket_ctx
            .send_bytes(packet.serialize(PROTOCOL_VERSION).as_slice())
            .await;
    }
}

async fn subscribe_to_queues(
    socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>,
    subscribers: &MySbSubscribers,
) {
    for subscriber in subscribers.get_subscribers().await {
        let packet = TcpContract::Subscribe {
            topic_id: subscriber.topic_id,
            queue_id: subscriber.queue_id,
            queue_type: subscriber.queue_type,
        };

        socket_ctx
            .send_bytes(packet.serialize(PROTOCOL_VERSION).as_slice())
            .await;
    }
}

const PROTOCOL_VERSION: i32 = 3;

pub fn get_connection_attrs() -> ConnectionAttributes {
    let mut attr = ConnectionAttributes::new(PROTOCOL_VERSION);

    attr.versions
        .set_packet_version(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    attr
}
