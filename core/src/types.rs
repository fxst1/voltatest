use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::alarm::{AlarmDescriptor, AlarmTrigger, AlarmTriggerResult};

/// Main Alert data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Optional description (can be set from tauri)
    description: Option<String>,

    /// Reception timestamp
    timestamp: NaiveDateTime,

    /// Raw datas
    data: Vec<u8>
}

impl Alert {

    pub fn new(description: Option<String>, timestamp: NaiveDateTime, data: Vec<u8>) -> Self {
        Self {
            description,
            timestamp,
            data,
        }
    }

    pub fn get_ref_description(&self) -> &Option<String> {
        return &self.description
    }

    pub fn get_ref_timestamp(&self) -> &NaiveDateTime {
        return &self.timestamp
    }

    pub fn get_ref_data(&self) -> &Vec<u8> {
        return &self.data
    }

}



#[derive(Clone)]
pub struct RawData {
    /// Reception timestamp
    timestamp: NaiveDateTime,

    /// Raw datas
    data: Vec<u8>
}

impl RawData {
    pub fn new(timestamp: NaiveDateTime, data: Vec<u8>) -> Self {
        Self {
            timestamp,
            data
        }
    }

    pub fn get_ref_timestamp(&self) -> &NaiveDateTime {
        &self.timestamp
    }

    pub fn get_ref_data(&self) -> &Vec<u8> {
        &self.data
    }
}

pub struct Alarm {
    trigger: Box<dyn AlarmTrigger>,
    descriptor: AlarmDescriptor,
}

impl Alarm {
    pub fn descriptor(&self) -> &AlarmDescriptor {
        &self.descriptor
    }
}

impl AlarmTrigger for Alarm {
    fn on_data(&mut self, raw: RawData) -> AlarmTriggerResult {
        self.trigger.on_data(raw)
    }
}