use tokio::sync::mpsc::UnboundedSender;

use crate::date_utils::MyDateTime;

use super::{LogType, MySbClientLogEvent, MySbLoggerReader};

pub struct MySbLogger {
    tx: Option<UnboundedSender<MySbClientLogEvent>>,
}

impl MySbLogger {
    pub fn new() -> Self {
        Self { tx: None }
    }

    pub fn get_reader(&mut self) -> MySbLoggerReader {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.tx = Some(tx);
        MySbLoggerReader::new(rx)
    }

    pub fn write_log(
        &self,
        log_type: LogType,
        process: String,
        message: String,
        context: Option<String>,
    ) {
        if let Some(tx) = &self.tx {
            let result = tx.send(MySbClientLogEvent {
                log_type,
                process,
                message,
                context,
            });

            if let Err(err) = result {
                println!("Somehow we could not send log event to sender. Err:{}", err);
            }
        } else {
            println!("{} {:?}", MyDateTime::utc_now().to_iso_string(), log_type);
            println!("Process: {}", process);
            println!("Message: {}", message);

            if let Some(ctx) = context {
                println!("Context: {}", ctx);
            }
        }
    }
}
