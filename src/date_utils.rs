use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub struct MyDateTime {
    pub micros: i64,
}

impl MyDateTime {
    pub fn utc_now() -> Self {
        let micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        Self { micros }
    }

    pub fn to_iso_string(&self) -> String {
        let d = UNIX_EPOCH + Duration::from_micros(self.micros as u64);

        let datetime = DateTime::<Utc>::from(d);

        return datetime.to_rfc3339();
    }

    pub fn get_duration_from(&self, before: MyDateTime) -> Duration {
        let dur = self.micros - before.micros;

        if dur < 0 {
            return Duration::from_micros(0);
        }

        Duration::from_micros(dur as u64)
    }

    pub fn get_duration_from_micros(&self, before_micros: i64) -> Duration {
        let dur = self.micros - before_micros;

        if dur < 0 {
            return Duration::from_micros(0);
        }

        Duration::from_micros(dur as u64)
    }
}
