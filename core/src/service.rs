use chrono::Utc;
use tokio::sync::mpsc;

use crate::alarm::AlarmDescriptor;
use crate::alarm::manager::AlarmManager;
use crate::error::VoltaTestError;
use crate::repo::{AlarmData, AlertData, AlertRepository};
use crate::types::{Alert, RawData};

pub struct AlertService {
    repo: Box<dyn AlertRepository>,
    alarm_manager: AlarmManager,
    alert_tx: mpsc::Sender<AlertData>,
}

impl AlertService {
    pub fn new(
        repo: Box<dyn AlertRepository>,
        alarm_manager: AlarmManager,
    ) -> (Self, mpsc::Receiver<AlertData>) {
        let (alert_tx, alert_rx) = mpsc::channel(32);
        (Self { repo, alarm_manager, alert_tx }, alert_rx)
    }

    pub fn create_alert(&mut self, alert: Alert) -> Result<AlertData, VoltaTestError> {
        self.repo.create(alert)
    }

    pub fn read_alert(&self, id: String) -> Result<Option<AlertData>, VoltaTestError> {
        self.repo.read(id)
    }

    pub fn delete_alert(&mut self, id: String) -> Result<(), VoltaTestError> {
        self.repo.delete(id)
    }

    pub fn list_alerts(&self, previous_id: Option<String>) -> Result<Vec<AlertData>, VoltaTestError> {
        self.repo.list(previous_id)
    }

    /// Evaluation alarms from manager and raise alerts
    pub fn evaluate_received_data(&mut self, data: Vec<u8>) -> Result<(), VoltaTestError> {
        let timestamp = Utc::now().naive_utc();
        let raw = RawData::new(timestamp, data.clone());

        let triggered = self.alarm_manager.evaluate_alarms(&raw)?;
        for alarm_data in triggered {
            let description = Some(alarm_data.get_ref_value().description.clone());
            let alert = Alert::new(description, timestamp, data.clone());
            let alert_data = self.repo.create(alert)?;
            self.alert_tx.try_send(alert_data).ok();
        }

        Ok(())
    }

    pub fn create_alarm(&mut self, descriptor: AlarmDescriptor) -> Result<AlarmData, VoltaTestError> {
        self.alarm_manager.create_alarm(descriptor)
    }

    pub fn read_alarm(&self, id: String) -> Result<Option<AlarmData>, VoltaTestError> {
        self.alarm_manager.read_alarm(id)
    }

    pub fn delete_alarm(&mut self, id: String) -> Result<(), VoltaTestError> {
        self.alarm_manager.delete_alarm(id)
    }

    pub fn list_alarms(&self, previous_id: Option<String>) -> Result<Vec<AlarmData>, VoltaTestError> {
        self.alarm_manager.list_alarms(previous_id)
    }
}
