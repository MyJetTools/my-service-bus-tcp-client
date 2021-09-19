use my_service_bus_tcp_shared::{ConnectionAttributes, PacketVersions};
use tokio::sync::RwLock;

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::date_utils::MyDateTime;

use super::SocketContextData;

pub struct SocketConnection {
    pub data: RwLock<SocketContextData>,
    pub id: i64,
    pub attr: ConnectionAttributes,
}
const PROTOCOL_VERSION: i32 = 2;

impl SocketConnection {
    pub fn new(id: i64, write_socket: WriteHalf<TcpStream>) -> Self {
        let mut attr = ConnectionAttributes {
            protocol_version: PROTOCOL_VERSION,
            versions: PacketVersions::new(),
        };

        attr.versions
            .set_packet_version(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGE, 1);

        Self {
            id,
            attr,
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
        write_access.disconnect(self.id).await;
    }

    pub async fn increase_read_size(&self, size: usize) {
        let mut write_access = self.data.write().await;
        write_access.read_size += size;
    }

    pub async fn update_last_read_time(&self) {
        let mut write_access = self.data.write().await;
        write_access.last_read_time = MyDateTime::utc_now();
    }

    pub async fn send_data_to_socket_and_forget(&self, payload: &[u8]) {
        let result = self.send_data_to_socket(payload).await;

        if let Err(err) = result {
            println!("Can not send payload to socket {}. Reason {}", self.id, err);
        }
    }

    pub async fn send_data_to_socket(&self, payload: &[u8]) -> Result<(), String> {
        let mut write_access = self.data.write().await;

        if write_access.disconnected {
            return Err(format!(
                "Can not write to socket {}. It's disconnected",
                self.id
            ));
        }

        let write_result = write_access.write_socket.write_all(payload).await;

        match write_result {
            Ok(_) => {
                write_access.last_write_time = MyDateTime::utc_now();
                write_access.write_size += payload.len();
                return Ok(());
            }
            Err(err) => {
                write_access.disconnect(self.id).await;
                return Err(format!(
                    "Can not write to socket {}. Reason: {:?}",
                    self.id, err
                ));
            }
        }
    }
}
