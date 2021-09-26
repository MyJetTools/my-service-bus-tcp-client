use my_service_bus_tcp_shared::TcpContract;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    subscribers::{MySbDeliveryConfirmation, MySbDeliveryConfirmationEvent},
    tcp::SocketConnection,
};

pub async fn start(mut confirmations_receiver: UnboundedReceiver<MySbDeliveryConfirmationEvent>) {
    let mut current_connection = None;

    loop {
        let confirmation_event = confirmations_receiver.recv().await;

        if confirmation_event.is_none() {
            return println!("Somehow we can empty confirmation event");
        }

        let confirmation_event = confirmation_event.unwrap();

        match confirmation_event {
            MySbDeliveryConfirmationEvent::Connected(ctx) => {
                current_connection = Some(ctx);
            }
            MySbDeliveryConfirmationEvent::Disconnected(connection_id) => match &current_connection
            {
                Some(connection) => {
                    if connection.id != connection_id {
                        println!("Some how we got disconnect event for connection {}. But current connection is {}", connection_id, connection.id);
                    }

                    current_connection = None;
                }
                None => {
                    println!("Some how we got disconnect event for connection {}. But there is no connection", connection_id);
                }
            },
            MySbDeliveryConfirmationEvent::Confirmation(confirmation) => {
                if let Some(connection) = &current_connection {
                    if confirmation.connection_id == connection.id {
                        send_confirmation(connection.as_ref(), confirmation).await;
                    }
                }
            }
        }
    }
}

async fn send_confirmation(connection: &SocketConnection, confirmation: MySbDeliveryConfirmation) {
    if let Some(not_delivered) = &confirmation.not_delivered {
        let tcp_packet = TcpContract::ConfirmMessagesByNotDelivery {
            topic_id: confirmation.topic_id,
            queue_id: confirmation.queue_id,
            confirmation_id: confirmation.confirmation_id,
            not_delivered: not_delivered.get_snapshot(),
            packet_version: 0, //ToDO - Check WTF
        };

        connection
            .send_data_to_socket_and_forget(tcp_packet.serialize(&connection.attr).as_slice())
            .await;
    } else {
        let tcp_packet = TcpContract::NewMessagesConfirmation {
            topic_id: confirmation.topic_id,
            queue_id: confirmation.queue_id,
            confirmation_id: confirmation.confirmation_id,
        };

        connection
            .send_data_to_socket_and_forget(tcp_packet.serialize(&connection.attr).as_slice())
            .await;
    }
}
