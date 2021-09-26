use std::{collections::HashMap, sync::Arc};

use rust_extensions::TaskCompletion;

use crate::tcp::SocketConnection;

use super::PublishError;

pub struct PublishProcessByConnection {
    pub socket: Arc<SocketConnection>,
    pub requests: HashMap<i64, TaskCompletion<(), PublishError>>,
}

impl PublishProcessByConnection {
    pub fn new(socket: Arc<SocketConnection>) -> Self {
        Self {
            requests: HashMap::new(),
            socket,
        }
    }
}

impl Drop for PublishProcessByConnection {
    fn drop(&mut self) {
        for (_, mut task) in self.requests.drain() {
            task.set_error(PublishError::Disconnected);
        }
    }
}
