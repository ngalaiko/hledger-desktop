use std::time::SystemTime;

use chrono::Datelike;

pub struct State {
    pub year: i32,
}

impl Default for State {
    fn default() -> Self {
        let current_time = SystemTime::now();
        let datetime: chrono::DateTime<chrono::Local> = current_time.into();
        Self {
            year: datetime.year(),
        }
    }
}
