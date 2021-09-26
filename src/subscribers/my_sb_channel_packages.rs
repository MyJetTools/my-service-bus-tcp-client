use std::sync::Arc;

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use my_service_bus_tcp_shared::TcpContractMessage;

use crate::tcp::SocketConnection;

pub struct MySbDeliveryPackage {
    pub messages: Vec<TcpContractMessage>,
    pub confirmation_id: i64,
    pub connection_id: i64,
}

pub enum MySbDeliveryConfirmationEvent {
    Connected(Arc<SocketConnection>),
    Disconnected(i64),
    Confirmation(MySbDeliveryConfirmation),
}

pub struct MySbDeliveryConfirmation {
    pub topic_id: String,
    pub queue_id: String,
    pub confirmation_id: i64,
    pub connection_id: i64,
    pub not_delivered: Option<QueueWithIntervals>,
}
