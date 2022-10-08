use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::publishers::MySbPublishers;
use crate::subscribers::MySbSubscribers;

use crate::TcpClientData;
use my_service_bus_abstractions::publisher::{MySbMessageSerializer, MyServiceBusPublisher};
use my_service_bus_abstractions::subscriber::MySbCallback;
use my_service_bus_abstractions::subscriber::MySbMessageDeserializer;
use my_service_bus_abstractions::subscriber::Subscriber;
use my_service_bus_abstractions::subscriber::TopicQueueType;
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
    pub tcp_client: TcpClient,
    data: Arc<TcpClientData>,
}

impl MyServiceBusClient {
    pub fn new(
        app_name: &str,
        app_version: &str,
        settings: Arc<dyn MyServiceBusSettings + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let tcp_settings = TcpConnectionSettings::new(settings);

        let data = TcpClientData {
            publishers: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            logger,
            has_connection: Arc::new(AtomicBool::new(false)),
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
            client_version: get_client_version(),
        };

        Self {
            tcp_client: TcpClient::new(TCP_CLIENT_NAME.to_string(), Arc::new(tcp_settings)),
            data: Arc::new(data),
        }
    }

    pub async fn start(&self) {
        self.tcp_client
            .start(
                Arc::new(|| -> MySbTcpSerializer {
                    let attrs = super::new_connection_handler::get_connection_attrs();
                    MySbTcpSerializer::new(attrs)
                }),
                self.data.clone(),
                self.data.logger.clone(),
            )
            .await;
    }

    pub async fn get_publisher<TContract: MySbMessageSerializer>(
        &self,
        topic_id: String,
        do_retries: bool,
    ) -> MyServiceBusPublisher<TContract> {
        self.data
            .publishers
            .create_topic_if_not_exists(topic_id.clone())
            .await;
        MyServiceBusPublisher::new(
            topic_id,
            self.data.publishers.clone(),
            do_retries,
            self.data.logger.clone(),
        )
    }

    pub async fn subscribe<
        TContract: MySbMessageDeserializer<Item = TContract> + Send + Sync + 'static,
    >(
        &self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        callback: Arc<dyn MySbCallback<TContract> + Send + Sync + 'static>,
    ) {
        let subscriber: Subscriber<TContract> = Subscriber::new(
            topic_id.clone(),
            queue_id.clone(),
            queue_type,
            callback,
            self.data.logger.clone(),
            self.data.subscribers.clone(),
        );

        let subscriber = Arc::new(subscriber);
        self.data
            .subscribers
            .add(topic_id.clone(), queue_id.clone(), subscriber)
            .await;
    }

    pub fn has_connection(&self) -> bool {
        self.data
            .has_connection
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn get_client_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
