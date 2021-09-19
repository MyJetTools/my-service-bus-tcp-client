use std::sync::Arc;

use my_service_bus_tcp_shared::{SocketReader, TcpContract};

use crate::MySbPublisher;
use tokio::net::TcpStream;

use tokio::io::ReadHalf;

use super::SocketConnection;

pub async fn start_new(
    read_socket: ReadHalf<TcpStream>,
    socket_connection: Arc<SocketConnection>,
    publisher: Arc<MySbPublisher>,
    app_name: String,
    client_version: String,
) {
    super::new_connection::send_init(
        socket_connection.as_ref(),
        app_name.as_str(),
        client_version.as_str(),
        publisher.as_ref(),
    )
    .await;

    let mut socket_reader = SocketReader::new(read_socket);

    loop {
        socket_reader.start_calculating_read_size();
        let deserialize_result =
            TcpContract::deserialize(&mut socket_reader, &socket_connection.attr).await;

        socket_connection
            .increase_read_size(socket_reader.read_size)
            .await;

        match deserialize_result {
            Ok(tcp_contract) => {
                socket_connection.update_last_read_time().await;

                super::incoming_events::new_packet(
                    socket_connection.as_ref(),
                    publisher.as_ref(),
                    tcp_contract,
                )
                .await;
            }

            Err(err) => {
                println!(
                    "Can not deserialize packet for the socket {} with the reason {:?}",
                    socket_connection.id, err
                );

                socket_connection.disconnect().await;
                return;
            }
        }
    }
}
