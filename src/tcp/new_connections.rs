use std::{sync::Arc, time::Duration};

use crate::MySbPublisher;
use tokio::net::TcpStream;

use tokio::io;

use super::SocketConnection;

pub async fn start(
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

                let read_task = tokio::task::spawn(super::read_loop::start_new(
                    read_socket,
                    socket_connection.clone(),
                    publisher.clone(),
                    app_name.clone(),
                    client_version.clone(),
                ));

                super::ping_loop::start_new(socket_connection.clone(), ping_timeout).await;

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
