use tokio::sync::oneshot::Receiver;

#[derive(Clone, Copy, Debug)]
pub enum TaskEvent<OkResult: Copy, ErrorResult: Copy> {
    Ok(OkResult),
    Error(ErrorResult),
}

pub struct TaskCompletionAwaiter<OkResult: Copy, ErrorResult: Copy> {
    pub receiver: Receiver<TaskEvent<OkResult, ErrorResult>>,
}

impl<OkResult: Copy, ErrorResult: Copy> TaskCompletionAwaiter<OkResult, ErrorResult> {
    pub fn new(receiver: Receiver<TaskEvent<OkResult, ErrorResult>>) -> Self {
        Self { receiver }
    }

    pub async fn get_result(self) -> Result<OkResult, ErrorResult> {
        let result = self.receiver.await;

        match result {
            Ok(result) => match result {
                TaskEvent::Ok(ok) => return Ok(ok),
                TaskEvent::Error(err) => return Err(err),
            },
            Err(error) => panic!(
                "Can not recivev result for a task completion. Err: {:?}",
                error
            ),
        }
    }
}
