use tokio::sync::{
    oneshot::{self, Receiver, Sender},
    Mutex,
};

#[derive(Clone, Copy, Debug)]
pub enum TaskEvent<OkResult: Copy, ErrorResult: Copy> {
    Ok(OkResult),
    Error(ErrorResult),
}

#[derive(Debug)]
pub enum TaskError<ErrorResult: Copy> {
    Error(ErrorResult),
    FatalError(String),
}

pub struct TaskCompletionData<OkResult: Copy, ErrorResult: Copy> {
    pub sender: Option<Sender<TaskEvent<OkResult, ErrorResult>>>,
    pub receiver: Option<Receiver<TaskEvent<OkResult, ErrorResult>>>,
}

impl<OkResult: Copy, ErrorResult: Copy> TaskCompletionData<OkResult, ErrorResult> {
    pub fn new() -> Self {
        let (sender, receiver) = oneshot::channel();

        Self {
            sender: Some(sender),
            receiver: Some(receiver),
        }
    }

    pub async fn get_sender(&mut self) -> Option<Sender<TaskEvent<OkResult, ErrorResult>>> {
        let mut new_result = None;
        std::mem::swap(&mut new_result, &mut self.sender);
        new_result
    }

    pub async fn get_receiver(&mut self) -> Option<Receiver<TaskEvent<OkResult, ErrorResult>>> {
        let mut new_result = None;
        std::mem::swap(&mut new_result, &mut self.receiver);
        new_result
    }
}

pub struct TaskCompletion<OkResult: Copy, ErrorResult: Copy> {
    pub trigger: Mutex<TaskCompletionData<OkResult, ErrorResult>>,
}

impl<OkResult: Copy, ErrorResult: Copy> TaskCompletion<OkResult, ErrorResult> {
    pub fn new() -> Self {
        Self {
            trigger: Mutex::new(TaskCompletionData::new()),
        }
    }

    async fn get_sender(&self) -> Option<Sender<TaskEvent<OkResult, ErrorResult>>> {
        let mut access = self.trigger.lock().await;
        access.get_sender().await
    }

    async fn get_receiver(&self) -> Option<Receiver<TaskEvent<OkResult, ErrorResult>>> {
        let mut access = self.trigger.lock().await;
        access.get_receiver().await
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

    pub async fn get_result(&self) -> Result<OkResult, TaskError<ErrorResult>> {
        let receiver = self.get_receiver().await;

        match receiver {
            Some(receiver) => match receiver.await {
                Ok(result) => match result {
                    TaskEvent::Ok(ok) => Ok(ok),
                    TaskEvent::Error(err) => Err(TaskError::Error(err)),
                },
                Err(error) => Err(TaskError::FatalError(format!(
                    "Error recieving result: {:?}",
                    error
                ))),
            },
            None => return Err(TaskError::FatalError("Task result cant not be recieved. Looks like it's a second time we are trying to get task result".to_string())),
        }
    }
}
