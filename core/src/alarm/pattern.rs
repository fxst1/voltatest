use serde::{Deserialize, Serialize};

use crate::alarm::{AlarmTrigger, AlarmTriggerResult};
use crate::types::RawData;

/// Trigger based on comparation
/// this logic can be extended to other operators (lt, gt, contains, between)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Operator {
    /// data == x
    Eq(Vec<u8>),
    /// data != x
    Neq(Vec<u8>),
}

/// Operator validation
impl Operator {
    pub fn is_valid(&self, data: &[u8]) -> bool {
        match self {
            Operator::Eq(v) => data == v.as_slice(),
            Operator::Neq(v) => data != v.as_slice(),
        }
    }
}

/// Pattern data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlarmPatternData {
    pub operator: Operator,
}

impl AlarmPatternData {
    pub fn new(operator: Operator) -> Self {
        Self { operator }
    }
}

impl AlarmTrigger for AlarmPatternData {
    fn on_data(&mut self, raw: RawData) -> AlarmTriggerResult {
        let triggered = self.operator.is_valid(raw.get_ref_data());
        (triggered, false).into()
    }
}
