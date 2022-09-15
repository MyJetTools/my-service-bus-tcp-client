#[async_trait::async_trait]
pub trait MyServiceBusSettings {
    async fn get_host_port(&self) -> String;
}
