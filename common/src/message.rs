use std::fmt;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub date_time: DateTime<Utc>,
    pub text: String,
}

impl fmt::Display for Message {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{} @ {}: {}",
               self.sender,
               self.date_time.with_timezone(&Local).format("%F %T"),
               self.text
        )
    }
}