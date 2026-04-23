use log::{debug, warn};

use crate::alarm::{AlarmDescriptor, AlarmKind, AlarmTrigger};
use crate::error::VoltaTestError;
use crate::repo::{AlarmData, AlarmRepository};
use crate::types::RawData;

/// In Memory alarm
struct ActiveAlarm {
    id: String,
    description: String,
    kind: AlarmKind,
}

/// Manages alarm lifecycle and evaluates incoming data against all active alarms.
pub struct AlarmManager {
    repo: Box<dyn AlarmRepository + Send>,
    active: Vec<ActiveAlarm>,
}

impl AlarmManager {

    /// Instanciate AlarmManager
    pub fn new(mut repo: Box<dyn AlarmRepository + Send>) -> Result<Self, VoltaTestError> {
        let stored = Self::load_all(&mut *repo)?;
        Ok(Self { repo, active: stored })
    }

    /// Setup active alarm on initiation
    fn load_all(repo: &mut dyn AlarmRepository) -> Result<Vec<ActiveAlarm>, VoltaTestError> {
        let mut active = Vec::new();
        let mut cursor: Option<String> = None;

        loop {

            let page = repo.list(cursor)?;
            if page.is_empty() {
                break;
            }

            cursor = page.last().map(|d| d.get_key());
            for alarm_data in page {
                let id = alarm_data.get_key();
                let descriptor = alarm_data.get_value();
                match AlarmKind::from_descriptor(&descriptor) {
                    Ok(kind) => active.push(ActiveAlarm {
                        id,
                        description: descriptor.description,
                        kind,
                    }),
                    Err(e) => warn!("Failed to load alarm {} it's descriptor: {}", id, e),
                }
            }
        }

        Ok(active)
    }

    /// Run all active alarms
    pub fn evaluate_alarms(&mut self, raw: &RawData) -> Result<Vec<AlarmData>, VoltaTestError> {
        
        let mut triggered = Vec::new();

        for (_, alarm) in self.active.iter_mut().enumerate() {
            let result = alarm.kind.on_data(raw.clone());

            let id          = alarm.id.clone();
            let description = alarm.description.clone();
            let configs     = alarm.kind.serialize_configs()?;
            let kind_id     = alarm.kind.kind_id().to_string();

            if result.update_required() {
                debug!("Alarm {} required change it's states", description);
                let descriptor = AlarmDescriptor::new(description.clone(), kind_id.clone(), configs.clone());
                self.repo.update(id.clone(), descriptor)?;
            }

            if result.trigger() {
                debug!("Alarm {} fired !", description);
                let descriptor = AlarmDescriptor::new(description, kind_id, configs);
                triggered.push(AlarmData::new(id, descriptor));
            }
            //results.push((idx, result.trigger(), result.update_required()));
        }

        Ok(triggered)
    }

    pub fn create_alarm(&mut self, descriptor: AlarmDescriptor) -> Result<AlarmData, VoltaTestError> {
        let kind = AlarmKind::from_descriptor(&descriptor)?;
        let alarm_data = self.repo.create(descriptor.clone())?;
        self.active.push(ActiveAlarm {
            id: alarm_data.get_key(),
            description: descriptor.description,
            kind,
        });
        Ok(alarm_data)
    }

    pub fn read_alarm(&self, id: String) -> Result<Option<AlarmData>, VoltaTestError> {
        self.repo.read(id)
    }

    pub fn update_alarm(&mut self, id: String, descriptor: AlarmDescriptor) -> Result<(), VoltaTestError> {
        let kind = AlarmKind::from_descriptor(&descriptor)?;
        self.repo.update(id.clone(), descriptor.clone())?;
        if let Some(alarm) = self.active.iter_mut().find(|a| a.id == id) {
            alarm.description = descriptor.description;
            alarm.kind = kind;
        }
        Ok(())
    }

    pub fn delete_alarm(&mut self, id: String) -> Result<(), VoltaTestError> {
        self.repo.delete(id.clone())?;
        self.active.retain(|a| a.id != id);
        Ok(())
    }

    pub fn list_alarms(&self, previous_id: Option<String>) -> Result<Vec<AlarmData>, VoltaTestError> {
        self.repo.list(previous_id)
    }
}
