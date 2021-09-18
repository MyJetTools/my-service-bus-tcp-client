use std::sync::Arc;

use my_service_bus_tcp_shared::TcpContract;
use tokio::sync::Mutex;

use crate::SocketContext;

pub struct MySbPublisherData {
    pub socket: Option<Arc<SocketContext>>,
    request_id: i64,
}

impl MySbPublisherData {
    pub fn new() -> Self {
        Self {
            socket: None,
            request_id: 0,
        }
    }
}

pub struct MySbPublisher {
    data: Mutex<MySbPublisherData>,
}

impl MySbPublisher {
    pub fn new() -> Self {
        let data = MySbPublisherData::new();
        Self {
            data: Mutex::new(data),
        }
    }
    pub async fn publish(&self, topic_id: &str, payload: Vec<u8>) -> Result<(), String> {
        let socket = self.get_current_socket().await;

        match socket {
            Some((socket, request_id)) => {
                let payload = TcpContract::Publish {
                    request_id,
                    persist_immediately: false,
                    data_to_publish: vec![payload],
                    topic_id: topic_id.to_string(),
                };

                let payload = payload.serialize(&socket.attr);

                socket.send_data_to_socket(payload.as_slice()).await?;
            }
            None => {
                return Err("Can not publish message. There is no connection".to_string());
            }
        }

        return Ok(()); //TODO - подключить TaskComplete;
    }

    async fn get_current_socket(&self) -> Option<(Arc<SocketContext>, i64)> {
        let mut write_access = self.data.lock().await;

        match &write_access.socket {
            Some(socket) => {
                let socket = socket.clone();
                write_access.request_id += 1;
                return Some((socket.clone(), write_access.request_id));
            }
            None => return None,
        }
    }

    pub async fn connect(&self, ctx: Arc<SocketContext>) {
        let mut write_access = self.data.lock().await;
        write_access.socket = Some(ctx);
    }

    pub async fn disconnect(&self) {
        let mut write_access = self.data.lock().await;
        write_access.socket = None;

        //TODO - обработать Disconnect
    }
}
