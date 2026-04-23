use serde::{Deserialize, Serialize};

use crate::alarm::clock::AlarmClockData;
use crate::alarm::pattern::AlarmPatternData;
use crate::error::VoltaTestError;
use crate::types::RawData;

pub struct AlarmTriggerResult(bool, bool);
impl AlarmTriggerResult {
    pub fn new(trigger: bool, update_repo: bool) -> Self {
        Self(trigger, update_repo)
    }

    /// Fire an alert
    pub fn trigger(&self) -> bool {
        self.0
    }

    /// Need to update alarm data store
    pub fn update_required(&self) -> bool {
        self.1
    }
}

impl From<(bool, bool)> for AlarmTriggerResult {
    fn from(value: (bool, bool)) -> Self {
        Self::new(value.0, value.1)
    }
}

pub trait AlarmTrigger {
    fn on_data(&mut self, raw: RawData) -> AlarmTriggerResult;
}

pub mod pattern;
pub mod clock;

pub enum AlarmKind {
    AlarmClock(AlarmClockData),
    Pattern(AlarmPatternData),
    Always()
}

impl AlarmKind {
    pub fn kind_id(&self) -> &str {
        match self {
            AlarmKind::AlarmClock(_) => "alarm-clock",
            AlarmKind::Pattern(_)    => "pattern",
            AlarmKind::Always()      => "always"
        }
    }

    /// Instanciate AlarmKind from a descriptor
    pub fn from_descriptor(descriptor: &AlarmDescriptor) -> Result<Self, VoltaTestError> {
        match descriptor.kind_id.as_str() {
            "alarm-clock" => {
                let data: AlarmClockData = serde_json::from_slice(&descriptor.configs)
                    .map_err(|e| VoltaTestError::alerting_error(format!("deserialize alarm-clock: {}", e)))?;
                Ok(AlarmKind::AlarmClock(data))
            }
            "pattern" => {
                let data: AlarmPatternData = serde_json::from_slice(&descriptor.configs)
                    .map_err(|e| VoltaTestError::alerting_error(format!("deserialize pattern: {}", e)))?;
                Ok(AlarmKind::Pattern(data))
            },
            "always"=> {
                Ok(AlarmKind::Always())
            }
            other => Err(VoltaTestError::alerting_error(format!("unknown alarm kind: {}", other))),
        }
    }

    /// Serialize the current trigger state back to a byte blob (for DB persistence).
    pub fn serialize_configs(&self) -> Result<Vec<u8>, VoltaTestError> {
        match self {
            AlarmKind::AlarmClock(d) => serde_json::to_vec(d)
                .map_err(|e| VoltaTestError::alerting_error(format!("serialize alarm-clock: {}", e))),
            AlarmKind::Pattern(d) => serde_json::to_vec(d)
                .map_err(|e| VoltaTestError::alerting_error(format!("serialize pattern: {}", e))),
            AlarmKind::Always() => serde_json::to_vec::<Option<String>>(&None)
                .map_err(|e| VoltaTestError::alerting_error(format!("serialize alarm-clock: {}", e))),
        }
    }
}

impl AlarmTrigger for AlarmKind {
    fn on_data(&mut self, raw: RawData) -> AlarmTriggerResult {
        match self {
            AlarmKind::AlarmClock(d) => d.on_data(raw),
            AlarmKind::Pattern(d)    => d.on_data(raw),
            AlarmKind::Always() => AlarmTriggerResult::new(true, false)
        }
    }
}

/// Alarm description for persitant storage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlarmDescriptor {
    /// Simple name
    pub description: String,
    /// AlarmKind::kind_id
    pub kind_id: String,
    /// Raw config depends on AlarmKind
    pub configs: Vec<u8>,
}

impl AlarmDescriptor {

    pub fn new(description: String, kind_id: String, configs: Vec<u8>) -> Self {
        Self { description, kind_id, configs }
    }

    pub fn get_ref_description(&self) -> &String {
        &self.description
    }

    pub fn get_ref_kind_id(&self) -> &String {
        &self.kind_id
    }

    pub fn get_ref_configs(&self) -> &Vec<u8> {
        &self.configs
    }
}

/// Alarm manager (alarm logic executor)
pub mod manager;
