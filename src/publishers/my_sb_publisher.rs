use std::sync::Arc;

use tokio::sync::Mutex;

use super::{MySbPublisherData, PublishError, PublishProcessByConnection};
use crate::tcp::SocketConnection;
use rust_extensions::TaskCompletionAwaiter;

pub struct MySbPublishers {
    data: Mutex<MySbPublisherData>,
}

impl MySbPublishers {
    pub fn new() -> Self {
        let data = MySbPublisherData::new();
        Self {
            data: Mutex::new(data),
        }
    }
    pub async fn publish(&self, topic_id: &str, payload: Vec<u8>) -> Result<(), PublishError> {
        let awaiter: TaskCompletionAwaiter<(), PublishError>;
        {
            let mut write_access = self.data.lock().await;
            awaiter = write_access.publish_to_socket(topic_id, payload).await?;
        }
        awaiter.get_result().await?;

        return Ok(());
    }

    pub async fn publish_confirmed(&self, connection_id: i64, request_id: i64) {
        let mut write_access = self.data.lock().await;
        write_access.confirm(connection_id, request_id).await;
    }

    pub async fn new_connection(&self, ctx: Arc<SocketConnection>) {
        let mut write_access = self.data.lock().await;

        if let Some(current_connection) = &write_access.connection {
            panic!("We are trying to insert new connection with Id {}, but we have connection {} not disconnected", 
            ctx.id,  current_connection.socket.id,);
        }

        write_access.connection = Some(PublishProcessByConnection::new(ctx));
    }

    pub async fn disconnect(&self, connection_id: i64) {
        let mut write_access = self.data.lock().await;

        if let Some(current_connection) = &write_access.connection {
            if current_connection.socket.id != connection_id {
                panic!("We are trying to disconnect connection with Id {}, but currect connection has id {}", connection_id, current_connection.socket.id);
            }
        }

        write_access.connection = None;
    }

    pub async fn create_topic_if_not_exists(&self, topic_id: String) {
        let mut write_access = self.data.lock().await;
        write_access.topics_to_create.insert(topic_id, 0);
    }

    pub async fn get_topics_to_create(&self) -> Vec<String> {
        let mut result = Vec::new();
        let write_access = self.data.lock().await;

        for topic_id in write_access.topics_to_create.keys() {
            result.push(topic_id.to_string());
        }

        result
    }
}
