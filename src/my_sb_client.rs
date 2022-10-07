use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::subscribers::{MySbSubscribers, SubscriberCallback};
use crate::tcp::IncomingTcpEvents;
use crate::MySbPublishers;
use my_service_bus_abstractions::{MySbMessageSerializer, MyServiceBusPublisher};
use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_shared::MySbTcpSerializer;
use my_tcp_sockets::{TcpClient, TcpClientSocketSettings};
use rust_extensions::Logger;

use super::MyServiceBusSettings;

const TCP_CLIENT_NAME: &str = "MySbTcpClient";

struct TcpConnectionSettings {
    my_sb_settings: Arc<dyn MyServiceBusSettings + Send + Sync + 'static>,
}

impl TcpConnectionSettings {
    pub fn new(my_sb_settings: Arc<dyn MyServiceBusSettings + Send + Sync + 'static>) -> Self {
        Self { my_sb_settings }
    }
}

#[async_trait::async_trait]
impl TcpClientSocketSettings for TcpConnectionSettings {
    async fn get_host_port(&self) -> String {
        self.my_sb_settings.get_host_port().await
    }
}

pub struct MyServiceBusClient {
    pub app_name: String,
    pub client_version: String,
    pub publishers: Arc<MySbPublishers>,
    pub subscribers: Arc<MySbSubscribers>,
    pub tcp_client: TcpClient,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
    has_connection: Arc<AtomicBool>,
}

impl MyServiceBusClient {
    pub fn new(
        app_name: &str,
        settings: Arc<dyn MyServiceBusSettings + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let tcp_settings = TcpConnectionSettings::new(settings);

        Self {
            app_name: app_name.to_string(),
            client_version: get_client_version(),
            publishers: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            tcp_client: TcpClient::new(TCP_CLIENT_NAME.to_string(), Arc::new(tcp_settings)),
            logger,
            has_connection: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        self.tcp_client
            .start(
                Arc::new(|| -> MySbTcpSerializer {
                    let attrs = crate::tcp::new_connection_handler::get_connection_attrs();
                    MySbTcpSerializer::new(attrs)
                }),
                Arc::new(IncomingTcpEvents::new(self, self.has_connection.clone())),
                self.logger.clone(),
            )
            .await;
    }

    pub async fn get_publisher<TContract>(
        &self,
        topic_name: String,
        serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
    ) -> MyServiceBusPublisher<TContract> {
        my_service_bus_abstractions::MyServiceBusPublisher::new(
            topic_name,
            self.publishers.clone(),
            serializer,
        )
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

    pub fn has_connection(&self) -> bool {
        self.has_connection
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn get_client_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
