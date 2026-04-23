use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::alarm::AlarmTriggerResult;

/// Single time triggering at defined datetime
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlarmClockData {
    /// When it triggers
    pub trigger_at: NaiveDateTime,
    /// Already fired
    pub passed: bool,
}

impl AlarmClockData {
    pub fn new(trigger_at: NaiveDateTime) -> Self {
        Self { trigger_at, passed: false }
    }
}

impl super::AlarmTrigger for AlarmClockData {
    fn on_data(&mut self, raw: crate::types::RawData) -> AlarmTriggerResult {
        // Avoid trigger multiple times
        if self.passed {
            return (false, false).into();
        }
        let fires = raw.get_ref_timestamp() >= &self.trigger_at;
        if fires {
            self.passed = true;
        }
        // update_required only when state actually changed
        (fires, fires).into()
    }
}