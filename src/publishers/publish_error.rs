pub enum PublishError {
    NoConnectionToPublish,
    Disconnected,
    Other(String),
}

impl Into<PublishError> for String {
    fn into(self) -> PublishError {
        PublishError::Other(self)
    }
}
