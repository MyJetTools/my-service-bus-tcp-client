
Add to Cargo.toml file

```toml
[dependencies]
my-service-bus-tcp-client = { tag = "xxx", git = "https://github.com/MyJetTools/my-service-bus-tcp-client.git" }
my-service-bus-shared = { tag = "xxx", git = "https://github.com/MyJetTools/my-service-bus-shared.git" }

tokio = { version = "*", features = ["full"] }
tokio-util = "*"
```

Setup MySbConnection Settings Reader. MyServiceBus will use the trait MyServiceBusSettings each time connection has to be established...

```rust


#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    ....

    #[serde(rename = "MySb")]
    pub my_sb: String,
}

#[async_trait::async_trait]
impl MyServiceBusSettings for SettingsReader {
    async fn get_host_port(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.my_sb.clone()
    }
}


```


Code Example - how to publish messages:

```rust
use std::time::Duration;
use my_service_bus_tcp_client::MyServiceBusClient;

#[tokio::main]
async fn main() {

    let client = TcpClient::new(
        "test-app".to_string(),
        "127.0.0.1:6421".to_string(),
    );
    
    my_sb_connection.start().await;
    
    
    let data_to_publish = vec![MessageToPublish {
                                content: // Put payload of content here,
                                headers: // Put headers here,
                             }];

    let result = app_ctx
            .my_sb_connection
            .publish_chunk("topic_name".to_string(), data_to_publish)
            .await;

    if let Err(err) = error {
       println!("Publish error: {:?}", err);
    }
            
}
```

Code Example - how to subscribe and receive messages:

```rust
use async_trait::async_trait;
use my_service_bus_shared::queue::TopicQueueType;
use my_service_bus_tcp_client::{
    subscribers::{MessagesReader, SubscriberCallback},
    MyServiceBusClient,
};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {

    let client = TcpClient::new(
        "test-app".to_string(),
        "127.0.0.1:6421".to_string(),
    );

    my_sb_connection
        .subscribe(
            settings.topic_name.to_string(),
            "test-queue".to_string(),
            TopicQueueType::DeleteOnDisconnect,
            Arc::new(MySbSubscriber {}),
        )
        .await;

    my_sb_connection.start().await;

    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
            
}


pub struct MySbSubscriber {}

#[async_trait]
impl SubscriberCallback for MySbSubscriber {
    async fn new_events(&self, mut messages_reader: MessagesReader) {
        for msg in messages_reader.get_messages() {
            println!("{:?}", msg.headers);
            messages_reader.handled_ok(&msg);
        }
    }
}
```
