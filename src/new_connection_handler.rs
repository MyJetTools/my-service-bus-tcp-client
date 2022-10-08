use std::collections::HashMap;

use my_service_bus_tcp_shared::{ConnectionAttributes, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub const PROTOCOL_VERSION: i32 = 3;

pub async fn send_greeting(
    socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>,
    app_name: &str,
    app_version: &str,
    client_version: &str,
) {
    let greeting = TcpContract::Greeting {
        name: format!("{}:{};{}", app_name, app_version, client_version),
        protocol_version: PROTOCOL_VERSION,
    };

    let payload = greeting.serialize(PROTOCOL_VERSION);
    socket_ctx.send_bytes(payload.as_slice()).await;
}

pub async fn send_packet_versions(socket_ctx: &SocketConnection<TcpContract, MySbTcpSerializer>) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    let packet_versions = TcpContract::PacketVersions { packet_versions };
    let payload = packet_versions.serialize(PROTOCOL_VERSION);

    socket_ctx.send_bytes(payload.as_slice()).await;
}

pub fn get_connection_attrs() -> ConnectionAttributes {
    let mut attr = ConnectionAttributes::new(PROTOCOL_VERSION);

    attr.versions
        .set_packet_version(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    attr
}
