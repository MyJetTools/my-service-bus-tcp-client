use tokio::sync::RwLock;

use tokio::{
    io::{self, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::date_utils::MyDateTime;

pub struct SocketContext {
    pub data: RwLock<SocketContextData>,
}

impl SocketContext {
    pub fn new(write_socket: WriteHalf<TcpStream>) -> Self {
        Self {
            data: RwLock::new(SocketContextData::new(write_socket)),
        }
    }

    pub async fn get_last_read_time(&self) -> MyDateTime {
        let read_access = self.data.read().await;
        read_access.last_read_time
    }

    pub async fn disconnected(&self) -> bool {
        let read_access = self.data.read().await;
        read_access.disconnected
    }

    pub async fn disconnect(&self) {
        let mut write_access = self.data.write().await;
        write_access.disconnected = true;
    }
}

pub struct SocketContextData {
    pub write_socket: WriteHalf<TcpStream>,
    pub last_write_time: MyDateTime,
    pub last_read_time: MyDateTime,
    pub disconnected: bool,
}

impl SocketContextData {
    pub fn new(write_socket: WriteHalf<TcpStream>) -> Self {
        Self {
            write_socket,
            last_write_time: MyDateTime::utc_now(),
            last_read_time: MyDateTime::utc_now(),
            disconnected: true,
        }
    }
}
