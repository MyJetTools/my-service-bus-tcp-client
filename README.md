Add to Cargo.toml file

```toml
[dependencies]
my-service-bus-tcp-client = { branch = "main", git = "https://github.com/MyJetTools/my-service-bus-tcp-client.git" }

tokio = { version = "*", features = ["full"] }
tokio-util = "*"
```


Code Example:

```rust
use std::time::Duration;
use my_service_bus_tcp_client::MyServiceBusClient;

#[tokio::main]
async fn main() {
    let mut my_sb_connection = MyServiceBusClient::new(
        "127.0.0.1:6421",
        "rust-test-app",
        "1.0.0",
        Duration::from_secs(3),
        Duration::from_secs(3),
    );
    
    my_sb_connection.start().await;

    let error = my_sb_connection
                .publish("rust-test", payload)
                .await;

    if let Err(err) = error {
       println!("Publish error: {:?}", err);
    }
            
}
```
