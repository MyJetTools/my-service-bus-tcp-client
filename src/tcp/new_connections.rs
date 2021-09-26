use std::{sync::Arc, time::Duration};

use crate::subscribers::{ConfirmationSender, MySbDeliveryConfirmationEvent, MySbSubscribers};
use crate::{MySbLogger, MySbPublishers};
use tokio::net::TcpStream;

use tokio::io::{self, ReadHalf};

use super::SocketConnection;

pub async fn start(
    logger: Arc<MySbLogger>,
    host_port: String,
    app_name: String,
    client_version: String,
    ping_timeout: Duration,
    connect_timeout: Duration,
    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    confirmation_sender: Option<Arc<ConfirmationSender>>,
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

                if let Some(confirmation_sender) = &confirmation_sender {
                    confirmation_sender.send(MySbDeliveryConfirmationEvent::Connected(
                        socket_connection.clone(),
                    ))
                }

                super::incoming_events::connected(
                    socket_connection.clone(),
                    publisher.clone(),
                    subscribers.clone(),
                )
                .await;

                process_new_connection(
                    logger.clone(),
                    socket_connection.clone(),
                    ping_timeout,
                    read_socket,
                    publisher.clone(),
                    subscribers.clone(),
                    app_name.as_str(),
                    client_version.as_str(),
                )
                .await;

                if let Some(confirmation_sender) = &confirmation_sender {
                    confirmation_sender.send(MySbDeliveryConfirmationEvent::Disconnected(
                        socket_connection.id,
                    ))
                }

                super::incoming_events::disconnected(socket_connection, publisher.clone()).await;
            }
            Err(err) => {
                logger.write_log(
                    crate::logger::LogType::Error,
                    "Connect loop".to_string(),
                    format!(
                        "Can not connect to the socket {}. Err: {:?}",
                        host_port, err
                    ),
                    None,
                );
            }
        }
    }
}

async fn process_new_connection(
    logger: Arc<MySbLogger>,
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

    super::ping_loop::start_new(logger.clone(), socket_connection.clone(), ping_timeout).await;

    let read_result = read_task.await;
    socket_connection.disconnect().await;

    if let Err(err) = read_result {
        logger.write_log(
            crate::logger::LogType::Error,
            format!("Connection process {}", socket_connection.id),
            format!(
                "We have error exiting the read loop for the client socket {}",
                socket_connection.id,
            ),
            Some(format!("{:?}", err)),
        );
    }
}
