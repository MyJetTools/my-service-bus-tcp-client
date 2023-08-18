use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::publishers::MySbPublishers;
use crate::subscribers::MySbSubscribers;

use crate::TcpClientData;
use my_service_bus_abstractions::publisher::{
    MySbMessageSerializer, MyServiceBusPublisher, PublisherWithInternalQueue,
};
use my_service_bus_abstractions::subscriber::MySbMessageDeserializer;
use my_service_bus_abstractions::subscriber::Subscriber;
use my_service_bus_abstractions::subscriber::SubscriberCallback;
use my_service_bus_abstractions::subscriber::TopicQueueType;
use my_service_bus_abstractions::GetMySbModelTopicId;
use my_service_bus_tcp_shared::MySbTcpSerializer;
use my_tcp_sockets::{TcpClient, TcpClientSocketSettings};
use rust_extensions::{Logger, StrOrString};

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
        app_name: impl Into<StrOrString<'static>>,
        app_version: impl Into<StrOrString<'static>>,
        settings: Arc<dyn MyServiceBusSettings + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let tcp_settings = TcpConnectionSettings::new(settings);

        let data = TcpClientData {
            publishers: Arc::new(MySbPublishers::new()),
            subscribers: Arc::new(MySbSubscribers::new()),
            logger,
            has_connection: Arc::new(AtomicBool::new(false)),
            app_name: app_name.into(),
            app_version: app_version.into(),
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

    pub async fn get_publisher<TModel: MySbMessageSerializer + GetMySbModelTopicId>(
        &self,
        do_retries: bool,
    ) -> MyServiceBusPublisher<TModel> {
        let topic_id = TModel::get_topic_id();
        self.data
            .publishers
            .create_topic_if_not_exists(topic_id.to_string())
            .await;
        MyServiceBusPublisher::new(
            topic_id.to_string(),
            self.data.publishers.clone(),
            do_retries,
            self.data.logger.clone(),
        )
    }

    pub async fn get_publisher_with_internal_queue<
        TModel: MySbMessageSerializer + GetMySbModelTopicId,
    >(
        &self,
    ) -> PublisherWithInternalQueue<TModel> {
        let topic_id = TModel::get_topic_id();
        self.data
            .publishers
            .create_topic_if_not_exists(topic_id.to_string())
            .await;
        PublisherWithInternalQueue::new(
            topic_id.to_string(),
            self.data.publishers.clone(),
            self.data.logger.clone(),
        )
    }

    pub async fn subscribe<
        TModel: GetMySbModelTopicId + MySbMessageDeserializer<Item = TModel> + Send + Sync + 'static,
    >(
        &self,
        queue_id: impl Into<StrOrString<'static>>,
        queue_type: TopicQueueType,
        callback: Arc<dyn SubscriberCallback<TModel> + Send + Sync + 'static>,
    ) {
        let topic_id = TModel::get_topic_id();
        let queue_id: StrOrString<'static> = queue_id.into();

        let subscriber: Subscriber<TModel> = Subscriber::new(
            topic_id.into(),
            queue_id.clone(),
            queue_type,
            callback,
            self.data.logger.clone(),
            self.data.subscribers.clone(),
        );

        let subscriber = Arc::new(subscriber);
        self.data
            .subscribers
            .add(topic_id, queue_id.to_string(), subscriber)
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
