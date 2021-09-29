use std::collections::HashMap;

use my_service_bus_tcp_shared::TcpContract;

use crate::{subscribers::MySbSubscribers, MySbPublishers};

use super::SocketConnection;

pub async fn send_init(
    socket_ctx: &SocketConnection,
    app_name: &str,
    client_version: &str,
    publisher: &MySbPublishers,
    subscribers: &MySbSubscribers,
) {
    send_greeting(socket_ctx, app_name, client_version).await;
    send_packet_versions(socket_ctx).await;
    let topics = publisher.get_topics_to_create().await;
    create_topics_if_not_exists(socket_ctx, topics).await;
    subscribe_to_queues(socket_ctx, subscribers).await;
}

async fn send_greeting(socket_ctx: &SocketConnection, app_name: &str, client_version: &str) {
    let greeting = TcpContract::Greeting {
        name: format!("{};{}", app_name, client_version),
        protocol_version: socket_ctx.attr.protocol_version,
    };

    let payload = greeting.serialize(&socket_ctx.attr);
    socket_ctx
        .send_data_to_socket_and_forget(payload.as_slice())
        .await;
}

async fn send_packet_versions(socket_ctx: &SocketConnection) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGE, 1);

    let packet_versions = TcpContract::PacketVersions { packet_versions };
    let payload = packet_versions.serialize(&socket_ctx.attr);

    socket_ctx
        .send_data_to_socket_and_forget(payload.as_slice())
        .await;
}

async fn create_topics_if_not_exists(socket_ctx: &SocketConnection, topics: Vec<String>) {
    for topic_id in topics {
        let packet = TcpContract::CreateTopicIfNotExists { topic_id };

        socket_ctx
            .send_data_to_socket_and_forget(packet.serialize(&socket_ctx.attr).as_slice())
            .await;
    }
}

async fn subscribe_to_queues(socket_ctx: &SocketConnection, subscribers: &MySbSubscribers) {
    for subscriber in subscribers.get_subscribers().await {
        let packet = TcpContract::Subscribe {
            topic_id: subscriber.topic_id,
            queue_id: subscriber.queue_id,
            queue_type: subscriber.queue_type,
        };

        socket_ctx
            .send_data_to_socket_and_forget(packet.serialize(&socket_ctx.attr).as_slice())
            .await;
    }
}
