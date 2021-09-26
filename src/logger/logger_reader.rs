use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use super::MySbClientLogEvent;

pub struct MySbLoggerReader {
    rx: UnboundedReceiver<MySbClientLogEvent>,
}

impl MySbLoggerReader {
    pub fn new(rx: UnboundedReceiver<MySbClientLogEvent>) -> Self {
        Self { rx }
    }
    pub async fn get_next_line(&mut self) -> MySbClientLogEvent {
        loop {
            let line = self.rx.recv().await;

            if let Some(event) = line {
                return event;
            } else {
                println!("Some how we did not get the log line");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
