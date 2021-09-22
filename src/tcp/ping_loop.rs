use std::{sync::Arc, time::Duration};

use crate::date_utils::MyDateTime;

use super::SocketConnection;

pub async fn start_new(socket_connection: Arc<SocketConnection>, ping_timeout: Duration) {
    let ping_task = tokio::task::spawn(ping_loop(socket_connection.clone(), ping_timeout));

    let ping_result = ping_task.await;

    if let Err(err) = ping_result {
        println!(
            "We have error exiting the ping loop for the client socket {}.  Reason: {:?}",
            socket_connection.id, err
        );
        //TODO - Remove println!
    }
}

pub async fn ping_loop(socket_connection: Arc<SocketConnection>, ping_timeout: Duration) {
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
            socket_connection.disconnect().await;
            return;
        }
    }
}
