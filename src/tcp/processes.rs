use std::{sync::Arc, time::Duration};

use my_service_bus_tcp_shared::{SocketReader, TcpContract};

use crate::date_utils::MyDateTime;
use crate::MySbPublisher;
use tokio::net::TcpStream;

use tokio::io::{self, ReadHalf};

use super::SocketConnection;

pub async fn client_socket_loop(
    host_port: String,
    app_name: String,
    client_version: String,
    ping_timeout: Duration,
    connect_timeout: Duration,
    publisher: Arc<MySbPublisher>,
) {
    let mut socket_id = 0;
    loop {
        let connect_result = TcpStream::connect(host_port.as_str()).await;

        match connect_result {
            Ok(tcp_stream) => {
                socket_id += 1;

                let (read_socket, write_socket) = io::split(tcp_stream);

                let socket_connection = SocketConnection::new(socket_id, write_socket);

                let socket_connection = Arc::new(socket_connection);

                super::incoming_events::connected(socket_connection.clone(), publisher.as_ref())
                    .await;

                let read_task = tokio::task::spawn(socket_read_loop(
                    read_socket,
                    socket_connection.clone(),
                    publisher.clone(),
                    app_name.clone(),
                    client_version.clone(),
                ));

                let ping_task =
                    tokio::task::spawn(socket_ping_loop(socket_connection.clone(), ping_timeout));

                let ping_result = ping_task.await;

                if let Err(err) = ping_result {
                    println!(
                        "We have error exiting the ping loop for the client socket {}.  Reason: {:?}",
                        socket_id, err
                    );
                    //TODO - Remove println!
                }

                let read_result = read_task.await;

                if let Err(err) = read_result {
                    println!("We have error exiting the read loop for the client socket {}.  Reason: {:?}",
                                        socket_id, err
                                    );
                    //TODO - Remove println!
                }

                super::incoming_events::disconnected(
                    socket_connection.as_ref(),
                    publisher.as_ref(),
                )
                .await;
            }
            Err(err) => {
                println!(
                    "Can not connect to the socket {}. Err: {:?}",
                    host_port, err
                ); //ToDo - Plug loggs
            }
        }

        tokio::time::sleep(connect_timeout).await;
    }
}

async fn socket_read_loop(
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

async fn socket_ping_loop(socket_connection: Arc<SocketConnection>, ping_timeout: Duration) {
    let ping_packet = vec![my_service_bus_tcp_shared::tcp_message_id::PING];

    let disconnect_time_out = ping_timeout * 3;

    loop {
        tokio::time::sleep(ping_timeout).await;

        let now = MyDateTime::utc_now();
        let last_read_time = socket_connection.get_last_read_time().await;

        let last_incoming_delay = now.get_duration_from(last_read_time);

        if last_incoming_delay > disconnect_time_out {
            println!("No activity for the socket {}", socket_connection.id);
            socket_connection.disconnect().await;
            return;
        }

        let send_result = socket_connection
            .send_data_to_socket(ping_packet.as_slice())
            .await;

        if let Err(err) = send_result {
            println!(
                "Can not send ping packet to the socket {}. Err:{}",
                socket_connection.id, err
            );
            return;
        }
    }
}
