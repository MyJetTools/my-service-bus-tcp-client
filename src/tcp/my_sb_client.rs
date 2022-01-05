use std::collections::HashMap;
use std::sync::Arc;

use crate::publishers::PublishError;
use crate::subscribers::{MySbSubscribers, SubscriberCallback};
use crate::MySbPublishers;
use my_logger::GetMyLoggerReader;
use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::MySbTcpSerializer;
use my_tcp_sockets::TcpClient;

use super::incoming_events::IncomingTcpEvents;

const TCP_CLIENT_NAME: &str = "MySbTcpClient";

pub struct MyServiceBusClient {
    pub app_name: String,
    pub client_version: String,
    pub publishers: Arc<MySbPublishers>,
    pub subscribers: Arc<MySbSubscribers>,
    pub tcp_client: TcpClient,
}

pub struct MessageToPublish {
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}

impl MyServiceBusClient {
    pub fn new(host_port: &str, app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            client_version: get_client_version(),
            publishers: Arc::new(MySbPublishers::new()),
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
            publishers: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            tcp_client: TcpClient::new_with_logger_reader(
                TCP_CLIENT_NAME.to_string(),
                host_port.to_string(),
                get_logger,
            ),
        }
    }

    pub async fn start(&self) {
        self.tcp_client.start(
            Arc::new(|| -> MySbTcpSerializer {
                let attrs = super::new_connection_handler::get_connection_attrs();
                MySbTcpSerializer::new(attrs)
            }),
            Arc::new(IncomingTcpEvents::new(self)),
        );
    }

    pub async fn publish(
        &self,
        topic_id: &str,
        message: MessageToPublish,
    ) -> Result<(), PublishError> {
        self.publishers.publish(topic_id, message).await?;
        Ok(())
    }

    pub async fn publish_chunk(
        &self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
    ) -> Result<(), PublishError> {
        self.publishers.publish_chunk(topic_id, messages).await?;
        Ok(())
    }

    pub async fn create_topic_if_not_exists(&self, topic_id: String) {
        self.publishers.create_topic_if_not_exists(topic_id).await;
    }

    pub async fn subscribe(
        &self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        callback: Arc<dyn SubscriberCallback + Send + Sync + 'static>,
    ) {
        self.subscribers
            .add(topic_id.clone(), queue_id.clone(), queue_type, callback)
            .await;
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
