use std::sync::Arc;

use crate::publishers::PublishError;
use crate::subscribers::{MySbSubscribers, Subscriber};
use crate::MySbPublishers;
use my_logger::GetMyLoggerReader;
use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::MySbTcpSerializer;
use my_tcp_sockets::TcpClient;

const TCP_CLIENT_NAME: &str = "MySbTcpClient";

pub struct MyServiceBusClient {
    app_name: String,
    client_version: String,
    publisher: Arc<MySbPublishers>,
    subscribers: Arc<MySbSubscribers>,
    tcp_client: TcpClient,
}

impl MyServiceBusClient {
    pub fn new(host_port: &str, app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            client_version: get_client_version(),
            publisher: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            tcp_client: TcpClient::new(TCP_CLIENT_NAME.to_string(), host_port.to_string()),
        }
    }

    pub fn new_with_logger_reader<TGetMyLoggerReader: GetMyLoggerReader>(
        host_port: &str,
        app_name: &str,
        get_logger: &TGetMyLoggerReader,
    ) -> Self {
        Self {
            app_name: app_name.to_string(),
            client_version: get_client_version(),
            publisher: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            tcp_client: TcpClient::new_with_logger_reader(
                TCP_CLIENT_NAME.to_string(),
                host_port.to_string(),
                get_logger,
            ),
        }
    }

    pub async fn start(&self) {
        let socket_events_reader = self.tcp_client.start(Arc::new(|| -> MySbTcpSerializer {
            let attrs = super::new_connection_handler::get_connection_attrs();
            MySbTcpSerializer::new(attrs)
        }));

        tokio::task::spawn(super::incoming_events::start(
            socket_events_reader,
            self.publisher.clone(),
            self.subscribers.clone(),
            self.app_name.clone(),
            self.client_version.clone(),
        ));
    }

    pub async fn publish(&self, topic_id: &str, payload: Vec<u8>) -> Result<(), PublishError> {
        self.publisher.publish(topic_id, payload).await?;
        Ok(())
    }

    pub async fn publish_chunk(
        &self,
        topic_id: &str,
        payload: Vec<Vec<u8>>,
    ) -> Result<(), PublishError> {
        self.publisher.publish_chunk(topic_id, payload).await?;
        Ok(())
    }

    pub async fn create_topic_if_not_exists(&self, topic_id: String) {
        self.publisher.create_topic_if_not_exists(topic_id).await;
    }

    pub async fn subscribe(
        &self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    ) -> Subscriber {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.subscribers
            .add(topic_id.clone(), queue_id.clone(), queue_type, tx)
            .await;

        Subscriber::new(topic_id, queue_id, rx)
    }

    pub fn plug_logger<TGetMyLoggerReader: GetMyLoggerReader>(
        &mut self,
        get_logger: &TGetMyLoggerReader,
    ) {
        self.tcp_client.plug_logger(get_logger);
    }
}

fn get_client_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
