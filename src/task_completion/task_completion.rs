use tokio::sync::{
    oneshot::{Receiver, Sender},
    Mutex,
};

use super::{TaskCompletionAwaiter, TaskEvent};

pub struct TaskCompletion<OkResult: Copy, ErrorResult: Copy> {
    pub receiver: Option<Receiver<TaskEvent<OkResult, ErrorResult>>>,
    pub sender: Mutex<Option<Sender<TaskEvent<OkResult, ErrorResult>>>>,
}

impl<OkResult: Copy, ErrorResult: Copy> TaskCompletion<OkResult, ErrorResult> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        Self {
            receiver: Some(receiver),
            sender: Mutex::new(Some(sender)),
        }
    }

    async fn get_sender(&self) -> Option<Sender<TaskEvent<OkResult, ErrorResult>>> {
        let mut access = self.sender.lock().await;
        let mut new_result = None;
        std::mem::swap(&mut new_result, &mut access);
        new_result
    }

    async fn get_receiver(&mut self) -> Option<Receiver<TaskEvent<OkResult, ErrorResult>>> {
        let mut new_result = None;
        std::mem::swap(&mut new_result, &mut self.receiver);
        new_result
    }

    pub async fn set_ok(&self, result: OkResult) -> Result<(), String> {
        let sender = self.get_sender().await;

        match sender {
            Some(sender) => {
                let result = sender.send(TaskEvent::Ok(result));
                if let Err(_) = result {
                    return Err(format!("Can not set Ok result to the task completion. "));
                }
                return Ok(());
            }
            None => {
                return Err(format!(
                    "You are trying to set OK as a result for a second time"
                ))
            }
        }
    }

    pub async fn set_error(&self, result: ErrorResult) -> Result<(), String> {
        let sender = self.get_sender().await;

        match sender {
            Some(sender) => {
                let result = sender.send(TaskEvent::Error(result));
                if let Err(_) = result {
                    return Err(format!("Can not set Error result to the task completion. "));
                }
                return Ok(());
            }
            None => {
                return Err(format!(
                    "You are trying to set error as a result for a second time"
                ))
            }
        }
    }

    pub async fn get_awaiter(&mut self) -> Option<TaskCompletionAwaiter<OkResult, ErrorResult>> {
        let receiver = self.get_receiver().await?;
        Some(TaskCompletionAwaiter::new(receiver))
    }
}
