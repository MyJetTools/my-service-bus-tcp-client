Add to Cargo.toml file

```toml
[dependencies]
my-service-bus-tcp-client = { branch = "main", git = "https://github.com/MyJetTools/my-service-bus-tcp-client.git" }

tokio = { version = "*", features = ["full"] }
tokio-util = "*"
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
            .publish_chunk(app_ctx.settings.topic_name.as_str(), data_to_publish)
            .await;

    if let Err(err) = error {
       println!("Publish error: {:?}", err);
    }
            
}
```
