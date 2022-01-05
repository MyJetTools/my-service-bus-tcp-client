use std::collections::HashMap;

use my_service_bus_tcp_shared::{MessageToPublishTcpContract, TcpContract};
use my_tcp_sockets::ConnectionId;
use rust_extensions::{TaskCompletion, TaskCompletionAwaiter};

use crate::tcp::MessageToPublish;

use super::{PublishError, PublishProcessByConnection};

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

    fn get_next_request_id(&mut self) -> i64 {
        self.request_id += 1;
        return self.request_id;
    }

    pub async fn publish_to_socket(
        &mut self,
        topic_id: &str,
        messages: Vec<MessageToPublish>,
    ) -> Result<TaskCompletionAwaiter<(), PublishError>, PublishError> {
        if self.connection.is_none() {
            return Err(PublishError::NoConnectionToPublish);
        }
        let request_id = self.get_next_request_id();

        let connection = self.connection.as_mut().unwrap();

        let mut data_to_publish = Vec::new();

        for msg in messages {
            data_to_publish.push(MessageToPublishTcpContract {
                headers: msg.headers,
                content: msg.content,
            })
        }

        let payload = TcpContract::Publish {
            request_id,
            persist_immediately: false,
            data_to_publish,
            topic_id: topic_id.to_string(),
        };

        if !connection.socket.send(payload).await {
            return Err(PublishError::Other(format!(
                "Can not send data to connection {}",
                connection.socket.id,
            )));
        }

        let mut task = TaskCompletion::new();
        let awaiter = task.get_awaiter();

        connection.requests.insert(request_id, task);

        Ok(awaiter)
    }

    pub async fn confirm(&mut self, connection_id: ConnectionId, request_id: i64) {
        if self.connection.is_none() {
            panic!(
                "Can not confirm publish for connection with id {} and request_id {}. No Active Connection",
                connection_id, request_id
            );
        }

        let connection = self.connection.as_mut().unwrap();

        if connection.socket.id != connection_id {
            panic!(
                "We are handling publish confirmation for connection_id {}. But we found active connection id {}",
                connection.socket.id, connection_id
            );
        }

        let request = connection.requests.remove(&request_id);

        if request.is_none() {
            panic!(
                "We are handling publish confirmation for connection_id {} with request_id {}. But now request with that ID",
                 connection_id, request_id
            );
        }

        request.unwrap().set_ok(());
    }
}
