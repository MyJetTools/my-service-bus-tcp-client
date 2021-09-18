use my_service_bus_tcp_shared::{SocketReader, TcpContract};
use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc;
use tokio::{
    io::{self, ReadHalf},
    net::TcpStream,
};

use crate::date_utils::MyDateTime;
use crate::{MySbPublisher, SocketContext};

pub enum MyServiceBusEvent<TcpContract> {
    Connect(i64),
    Disconnect(i64),
    Packet { socket_id: i64, event: TcpContract },
}

pub struct MyServiceBusClient {
    host_port: String,
    app_name: String,
    clinet_version: String,
    connect_timeout: Duration,
    ping_timeout: Duration,
    packet_callback: Arc<mpsc::Sender<MyServiceBusEvent<TcpContract>>>,
    publisher: Arc<MySbPublisher>,
}

impl MyServiceBusClient {
    pub fn new(
        host_port: &str,
        app_name: &str,
        clinet_version: &str,
        connect_timeout: Duration,
        ping_timeout: Duration,
        packet_callback: mpsc::Sender<MyServiceBusEvent<TcpContract>>,
    ) -> Self {
        Self {
            host_port: host_port.to_string(),
            app_name: app_name.to_string(),
            clinet_version: clinet_version.to_string(),
            connect_timeout,
            ping_timeout,
            packet_callback: Arc::new(packet_callback),
            publisher: Arc::new(MySbPublisher::new()),
        }
    }

    pub fn start(&self) {
        tokio::task::spawn(client_socket_loop(
            self.host_port.to_string(),
            self.app_name.to_string(),
            self.clinet_version.to_string(),
            self.ping_timeout,
            self.connect_timeout,
            self.packet_callback.clone(),
            self.publisher.clone(),
        ));
    }

    pub async fn publish(&self, topic_id: &str, payload: Vec<u8>) -> Result<(), String> {
        self.publisher.publish(topic_id, payload).await?;
        Ok(())
    }
}

async fn client_socket_loop(
    host_port: String,
    app_name: String,
    client_version: String,
    ping_timeout: Duration,
    connect_timeout: Duration,
    packet_callback: Arc<mpsc::Sender<MyServiceBusEvent<TcpContract>>>,
    publisher: Arc<MySbPublisher>,
) {
    let mut socket_id = 0;
    loop {
        let connect_result = TcpStream::connect(host_port.as_str()).await;

        match connect_result {
            Ok(tcp_stream) => {
                socket_id += 1;

                let (read_socket, write_socket) = io::split(tcp_stream);

                let socket_ctx = SocketContext::new(socket_id, write_socket);

                let socket_ctx = Arc::new(socket_ctx);

                let connect_sent_result = packet_callback
                    .as_ref()
                    .send(MyServiceBusEvent::Connect(socket_id))
                    .await;

                if let Err(err) = connect_sent_result {
                    println!("Can not send connect event to tcp mscp. Reason: {}", err); //TODO - Remove println!
                    continue;
                }

                publisher.connect(socket_ctx.clone()).await;

                let read_task = tokio::task::spawn(socket_read_loop(
                    read_socket,
                    socket_ctx.clone(),
                    packet_callback.clone(),
                ));

                let ping_task = tokio::task::spawn(socket_ping_loop(socket_ctx, ping_timeout));

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

                let disconnect_sent_result = packet_callback
                    .as_ref()
                    .send(MyServiceBusEvent::Disconnect(socket_id))
                    .await;

                publisher.disconnect().await;

                if let Err(err) = disconnect_sent_result {
                    println!("Can not send disconnect event to tcp mscp. Reason: {}", err); //TODO - Remove println!
                    panic!("Disconnect send error. Reason: {}", err);
                }
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
    socket_ctx: Arc<SocketContext>,
    packet_callback: Arc<mpsc::Sender<MyServiceBusEvent<TcpContract>>>,
) {
    send_init_data_to_socket(socket_ctx.as_ref()).await;

    let mut socket_reader = SocketReader::new(read_socket);

    loop {
        socket_reader.start_calculating_read_size();
        let deserialize_result =
            TcpContract::deserialize(&mut socket_reader, &socket_ctx.attr).await;

        socket_ctx.increase_read_size(socket_reader.read_size).await;

        match deserialize_result {
            Ok(event) => {
                socket_ctx.update_last_read_time().await;

                let sent_result = packet_callback
                    .send(MyServiceBusEvent::Packet {
                        socket_id: socket_ctx.socket_id,
                        event,
                    })
                    .await;

                if let Err(err) = sent_result {
                    println!(
                        "Can not Send packet to the mscp for the socket {} with the reason {}",
                        socket_ctx.socket_id, err
                    );

                    socket_ctx.disconnect().await;
                    return;
                }
            }

            Err(err) => {
                println!(
                    "Can not deserialize packet for the socket {} with the reason {:?}",
                    socket_ctx.socket_id, err
                );

                socket_ctx.disconnect().await;
                return;
            }
        }
    }
}

async fn send_init_data_to_socket(socket_ctx: &SocketContext) {
    //TODO - Implement
}

async fn socket_ping_loop(socket_ctx: Arc<SocketContext>, ping_timeout: Duration) {
    let ping_packet = vec![my_service_bus_tcp_shared::tcp_message_id::PING];

    let disconnect_time_out = ping_timeout * 3;

    loop {
        tokio::time::sleep(ping_timeout).await;

        let now = MyDateTime::utc_now();
        let last_read_time = socket_ctx.get_last_read_time().await;

        let last_incoming_delay = now.get_duration_from(last_read_time);

        if last_incoming_delay > disconnect_time_out {
            println!("No activity for the socket {}", socket_ctx.socket_id);
            socket_ctx.disconnect().await;
            return;
        }

        let send_result = socket_ctx.send_data_to_socket(ping_packet.as_slice()).await;

        if let Err(err) = send_result {
            println!(
                "Can not send ping packet to the socket {}. Err:{}",
                socket_ctx.socket_id, err
            );
            return;
        }
    }
}
