#[derive(Debug)]
pub enum LogType {
    Info,
    Error,
    FatalError,
}
#[derive(Debug)]
pub struct MySbClientLogEvent {
    pub log_type: LogType,
    pub process: String,
    pub message: String,
    pub context: Option<String>,
}
