use std::collections::HashMap;

use my_service_bus_abstractions::{publisher::MessageToPublish, PublishError};
use my_service_bus_tcp_shared::TcpContract;
use rust_extensions::{TaskCompletion, TaskCompletionAwaiter};

use super::PublishProcessByConnection;

pub struct MySbPublisherData {
    request_id: i64,
    pub connection: Option<PublishProcessByConnection>,
    pub topics_to_create: HashMap<String, i32>,
}

impl MySbPublisherData {
    pub fn new() -> Self {
        Self {
            request_id: 0,
            connection: None,
            topics_to_create: HashMap::new(),
        }
    }

    pub fn get_next_request_id(&mut self) -> i64 {
        self.request_id += 1;
        return self.request_id;
    }

    pub async fn compile_publish_payload(
        &mut self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
    ) -> Result<(i64, TcpContract), PublishError> {
        if self.connection.is_none() {
            return Err(PublishError::NoConnectionToPublish);
        }

        let request_id = self.get_next_request_id();

        let mut data_to_publish = Vec::with_capacity(messages.len());

        for msg in messages {
            data_to_publish.push(MessageToPublish {
                headers: msg.headers,
                content: msg.content,
            })
        }

        let result = TcpContract::Publish {
            request_id,
            persist_immediately: false,
            data_to_publish,
            topic_id: topic_id.to_string(),
        };

        Ok((request_id, result))
    }

    pub async fn publish_to_socket(
        &mut self,
        tcp_contract: &TcpContract,
        request_id: i64,
    ) -> TaskCompletionAwaiter<(), PublishError> {
        let connection = self.connection.as_mut().unwrap();

        connection.socket.send_ref(tcp_contract).await;

        let mut task = TaskCompletion::new();
        let awaiter = task.get_awaiter();

        connection.requests.insert(request_id, task);

        awaiter
    }

    pub async fn confirm(&mut self, request_id: i64) {
        if let Some(connection) = self.connection.as_mut() {
            if let Some(mut request) = connection.requests.remove(&request_id) {
                request.set_ok(());
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.connection = None;
    }
}
