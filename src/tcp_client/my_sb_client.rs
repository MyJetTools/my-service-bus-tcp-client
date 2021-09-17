use std::{sync::Arc, time::Duration};

use async_trait::async_trait;

use tokio::sync::mpsc;
use tokio::{
    io::{self, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::date_utils::MyDateTime;

use super::SocketContext;

pub enum TcpEvent<TPacket: Send + 'static> {
    Connect(i64),
    Disconnect(i64),
    Packet(TPacket),
}

#[async_trait]
pub trait TTcpSerializer<TPacket: Send> {
    fn get_ping_packet(&self) -> Vec<u8>;
    async fn deserialize(&self, read: &ReadHalf<TcpStream>) -> Result<TPacket, String>;
}

pub struct MyTcpClient<
    TPacket: Send + 'static,
    TSer: TTcpSerializer<TPacket> + Copy + Send + 'static,
> {
    host_port: String,
    connect_timeout: Duration,
    ping_timeout: Duration,
    ser: TSer,
    packet_callback: Arc<mpsc::Sender<TcpEvent<TPacket>>>,
}

impl<TPacket: Send + 'static, TSer: TTcpSerializer<TPacket> + Copy + Send + 'static>
    MyTcpClient<TPacket, TSer>
{
    pub fn new(
        host_port: &str,
        connect_timeout: Duration,
        ping_timeout: Duration,
        ser: TSer,
        packet_callback: mpsc::Sender<TcpEvent<TPacket>>,
    ) -> Self {
        Self {
            host_port: host_port.to_string(),
            connect_timeout,
            ping_timeout,
            ser,
            packet_callback: Arc::new(packet_callback),
        }
    }

    pub fn start(&self) {
        let ser = self.ser;
        tokio::task::spawn(client_socket_loop(
            self.host_port.to_string(),
            self.ping_timeout,
            self.connect_timeout,
            ser,
            self.packet_callback.clone(),
        ));
    }
}

async fn client_socket_loop<
    TPacket: Send + 'static,
    TSer: TTcpSerializer<TPacket> + Copy + Send + 'static,
>(
    host_port: String,
    ping_timeout: Duration,
    connect_timeout: Duration,
    ser: TSer,
    packet_callback: Arc<mpsc::Sender<TcpEvent<TPacket>>>,
) {
    let mut socket_id = 0;
    loop {
        let connect_result = TcpStream::connect(host_port.as_str()).await;

        match connect_result {
            Ok(tcp_stream) => {
                socket_id += 1;

                let (read_socket, write_socket) = io::split(tcp_stream);

                let socket_ctx = SocketContext::new(write_socket);

                let socket_ctx = Arc::new(socket_ctx);

                let connect_sent_result = packet_callback
                    .as_ref()
                    .send(TcpEvent::Connect(socket_id))
                    .await;

                if let Err(err) = connect_sent_result {
                    println!("Can not send connect event to tcp mscp. Reason: {}", err); //TODO - Remove println!
                    continue;
                }

                let read_task = tokio::task::spawn(socket_read_loop(
                    socket_id,
                    read_socket,
                    socket_ctx.clone(),
                    ser,
                    packet_callback.clone(),
                ));

                let ping_task =
                    tokio::task::spawn(socket_ping_loop(socket_id, socket_ctx, ping_timeout, ser));

                let ping_result = ping_task.await;

                if let Err(err) = ping_result {
                    println!(
                        "We had error exiting the ping loop for the client socket {}.  Reason: {:?}",
                        socket_id, err
                    );
                    //TODO - Remove println!
                }

                let read_result = read_task.await;

                if let Err(err) = read_result {
                    println!("We had error exiting the read loop for the client socket {}.  Reason: {:?}",
                                        socket_id, err
                                    );
                    //TODO - Remove println!
                }

                let disconnect_sent_result = packet_callback
                    .as_ref()
                    .send(TcpEvent::Connect(socket_id))
                    .await;

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

async fn socket_read_loop<
    TPacket: Send + 'static,
    TSer: TTcpSerializer<TPacket> + Send + Copy + 'static,
>(
    socket_id: i64,
    read_socket: ReadHalf<TcpStream>,
    socket_ctx: Arc<SocketContext>,
    ser: TSer,
    packet_callback: Arc<mpsc::Sender<TcpEvent<TPacket>>>,
) {
    loop {
        let deserialize_result = ser.deserialize(&read_socket).await;

        if let Err(err) = deserialize_result {
            println!(
                "Can not deserialize packet for the socket {} with the reason {}",
                socket_id, err
            );

            socket_ctx.disconnect().await;
            return;
        }

        let packet = deserialize_result.unwrap();

        let sent_result = packet_callback.send(TcpEvent::Packet(packet)).await;

        if let Err(err) = sent_result {
            println!(
                "Can not Send packet to the mscp for the socket {} with the reason {}",
                socket_id, err
            );

            socket_ctx.disconnect().await;
            return;
        }
    }
}

async fn socket_ping_loop<
    TPacket: Send + 'static,
    TSer: TTcpSerializer<TPacket> + Send + Copy + 'static,
>(
    socket_id: i64,
    socket_ctx: Arc<SocketContext>,
    ping_timeout: Duration,
    ser: TSer,
) {
    let ping_packet = ser.get_ping_packet();

    let disconnect_time_out = ping_timeout * 3;

    loop {
        tokio::time::sleep(ping_timeout).await;

        let now = MyDateTime::utc_now();
        let last_read_time = socket_ctx.get_last_read_time().await;

        let last_incoming_delay = now.get_duration_from(last_read_time);

        let mut write_access = socket_ctx.data.write().await;

        if last_incoming_delay > disconnect_time_out {}

        let write_result = write_access
            .write_socket
            .write_all(ping_packet.as_slice())
            .await;

        if let Err(err) = write_result {
            println!(
                "Can not send ping packet to the socket {}. Err:{:?}",
                socket_id, err
            ); //TODO - Remove println

            write_access.disconnected = true;
            break;
        }
    }
}
