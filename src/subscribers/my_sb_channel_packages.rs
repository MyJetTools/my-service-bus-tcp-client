use std::sync::Arc;

use my_service_bus_tcp_shared::{TcpContract, TcpContractMessage};
use my_tcp_sockets::tcp_connection::SocketConnection;

pub struct MySbDeliveryPackage {
    pub messages: Vec<TcpContractMessage>,
    pub confirmation_id: i64,
    pub connection: Arc<SocketConnection<TcpContract>>,
}
