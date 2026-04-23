use serde::{Deserialize, Serialize};

use crate::{alarm::AlarmDescriptor, error::VoltaTestError, types::Alert};

#[derive(Debug, Serialize, Clone)]
pub struct EntityData<T: Serialize + for <'a> Deserialize<'a> + Clone>
(String, T);

impl <T> EntityData<T>
    where T: Serialize + for <'a> Deserialize<'a> + Clone
{
    pub fn new(id: String, value: T) -> Self {
        Self(id, value)
    }

    pub fn get_key(&self) -> String {
        self.0.clone()
    }
    pub fn get_value(&self) -> T {
        self.1.clone()
    }

    pub fn get_ref_key(&self) -> &String {
        &self.0
    }
    pub fn get_ref_value(&self) -> &T {
        &self.1
    }
}

/// CRUD-based alert storage API
pub type AlertData = EntityData<Alert>;
pub type AlarmData = EntityData<AlarmDescriptor>;

pub trait AlertRepository {
    fn create(&mut self, alert: Alert) -> Result<AlertData, VoltaTestError>;
    fn read(&self, id: String) -> Result<Option<AlertData>, VoltaTestError>;
    fn update(&mut self, id: String, alert: Alert) -> Result<(), VoltaTestError>;
    fn delete(&mut self, id: String) -> Result<(), VoltaTestError>;
    fn list(&self, previous_id: Option<String>) -> Result<Vec<AlertData>, VoltaTestError>;
}

pub trait AlarmRepository {
    fn create(&mut self, alert: AlarmDescriptor) -> Result<AlarmData, VoltaTestError>;
    fn read(&self, id: String) -> Result<Option<AlarmData>, VoltaTestError>;
    fn update(&mut self, id: String, alert: AlarmDescriptor) -> Result<(), VoltaTestError>;
    fn delete(&mut self, id: String) -> Result<(), VoltaTestError>;
    fn list(&self, previous_id: Option<String>) -> Result<Vec<AlarmData>, VoltaTestError>;
}

pub mod sqlite;
