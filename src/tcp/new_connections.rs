use std::{sync::Arc, time::Duration};

use crate::subscribers::MySbSubscribers;
use crate::MySbPublishers;
use tokio::net::TcpStream;

use tokio::io::{self, ReadHalf};

use super::SocketConnection;

pub async fn start(
    host_port: String,
    app_name: String,
    client_version: String,
    ping_timeout: Duration,
    connect_timeout: Duration,
    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
) {
    let mut socket_id = 0;
    loop {
        tokio::time::sleep(connect_timeout).await;
        socket_id += 1;

        let connect_result = TcpStream::connect(host_port.as_str()).await;

        match connect_result {
            Ok(tcp_stream) => {
                let (read_socket, write_socket) = io::split(tcp_stream);

                let socket_connection = SocketConnection::new(socket_id, write_socket);

                let socket_connection = Arc::new(socket_connection);

                super::incoming_events::connected(
                    socket_connection.clone(),
                    publisher.clone(),
                    subscribers.clone(),
                )
                .await;

                process_new_connection(
                    socket_connection.clone(),
                    ping_timeout,
                    read_socket,
                    publisher.clone(),
                    subscribers.clone(),
                    app_name.as_str(),
                    client_version.as_str(),
                )
                .await;

                super::incoming_events::disconnected(socket_connection, publisher.clone()).await;
            }
            Err(err) => {
                println!(
                    "Can not connect to the socket {}. Err: {:?}",
                    host_port, err
                ); //ToDo - Plug loggs
            }
        }
    }
}

async fn process_new_connection(
    socket_connection: Arc<SocketConnection>,
    ping_timeout: Duration,
    read_socket: ReadHalf<TcpStream>,
    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    app_name: &str,
    client_version: &str,
) {
    let read_task = tokio::task::spawn(super::read_loop::start_new(
        read_socket,
        socket_connection.clone(),
        publisher,
        subscribers,
        app_name.to_string(),
        client_version.to_string(),
    ));

    //TODO -проверить что если мы спаникуем выше - мы выскочим и отсюда тоже
    super::ping_loop::start_new(socket_connection.clone(), ping_timeout).await;

    let read_result = read_task.await;
    socket_connection.disconnect().await;

    if let Err(err) = read_result {
        println!(
            "We have error exiting the read loop for the client socket {}.  Reason: {:?}",
            socket_connection.id, err
        );
        //TODO - Remove println!
    }
}
