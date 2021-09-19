use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::date_utils::MyDateTime;

pub struct SocketContextData {
    pub write_socket: WriteHalf<TcpStream>,
    pub last_write_time: MyDateTime,
    pub last_read_time: MyDateTime,
    pub read_size: usize,
    pub write_size: usize,
    pub disconnected: bool,
}

impl SocketContextData {
    pub fn new(write_socket: WriteHalf<TcpStream>) -> Self {
        Self {
            write_socket,
            last_write_time: MyDateTime::utc_now(),
            last_read_time: MyDateTime::utc_now(),
            disconnected: true,
            read_size: 0,
            write_size: 0,
        }
    }

    pub async fn disconnect(&mut self, connection_id: i64) {
        if self.disconnected {
            return;
        }

        self.disconnected = true;
        let shutdown_result = self.write_socket.shutdown().await;

        if let Err(err) = shutdown_result {
            println!(
                "Can not shutdown socket with id {}. Reason: {:?}",
                connection_id, err
            );
        }
    }
}
