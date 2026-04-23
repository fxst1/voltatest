use log::debug;
use rusqlite::{Connection};
use crate::{alarm::AlarmDescriptor, error::VoltaTestError, repo::{AlarmData, AlarmRepository}};

pub struct SqliteAlarmRepo {
    connection: Connection
}

/// Queries to execute at startup (ensure DB is ready)
const QUERIES: [&str;1] = [
    "
        CREATE TABLE IF NOT EXISTS alarm (
                alarm_id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT DEFAULT NULL,
                kind VARCHAR(32) NOT NULL,
                configs BLOB NOT NULL
        );
    ",
];

impl SqliteAlarmRepo {

    /// Open a connection from a file path
    pub fn from_path(path: &str) -> Result<Self, VoltaTestError> {
        let connection = Connection::open(path)
            .map_err(| connection_err | VoltaTestError::repository_error(format!("connection from path {}: {}", path, connection_err)))
            ?;

        let instance = Self { connection };
        instance.startup()?;

        Ok(instance)
    }

    /// Open a connection from memory, useful for testing
    pub fn from_memory() -> Result<Self, VoltaTestError> {
        let connection = Connection::open_in_memory()
            .map_err(| connection_err | VoltaTestError::repository_error(format!("connection from memory: {}", connection_err)))
            ?;

        let instance = Self { connection };
        instance.startup()?;

        Ok(instance)
    }

    /// Database setup
    fn startup(&self) -> Result<(), VoltaTestError> {
        for query in QUERIES {
            self.connection.execute(query, ())
                .map_err(|err | VoltaTestError::repository_error(format!("Startup error: {}", err)))
                ?;
        }
        Ok(())
    }
}


impl AlarmRepository for SqliteAlarmRepo {

    /// Create an alarm
    fn create(&mut self, alarm: AlarmDescriptor) -> Result<AlarmData, VoltaTestError> {
        self.connection.execute(
            "INSERT INTO alarm (
                    description,
                    kind,
                    configs
            ) VALUES (?1, ?2, ?3)",
            (alarm.get_ref_description(), alarm.get_ref_kind_id(), alarm.get_ref_configs()),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Create Alarm: {}", err)))
        ?;

        Ok(AlarmData::new(
            self.connection.last_insert_rowid().to_string(),
            alarm
        ))
    }

    /// Read an alarm
    fn read(&self, id: String) -> Result<Option<AlarmData>, VoltaTestError> {

        let mut stmt = self.connection.prepare("
            SELECT alarm_id, description, kind, configs
            FROM alarm
            WHERE alarm_id = ?1
            LIMIT 1
        ")
            .map_err(| err | VoltaTestError::repository_error(format!("read {}: {}", id, err)))
        ?;

        // Reading only one item
        let alarm_result = stmt.query_one([&id], |row| {
            let id: i64 = row.get(0)?;
            Ok(AlarmData::new(
                id.to_string(),
                AlarmDescriptor::new(
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                )
            ))
        });

        match alarm_result {
            Ok(val) => {
                Ok(Some(val))
            },
            Err(err) => {
                // Catch empty query result as None
                if let rusqlite::Error::QueryReturnedNoRows = err {
                    Ok(None)
                } else {
                    Err(VoltaTestError::repository_error(format!("read {}: {}", id, err)))
                }
            }
        }

    }

    fn update(&mut self, id: String, alarm: AlarmDescriptor) -> Result<(), VoltaTestError> {

        self.connection.execute(
            "UPDATE alarm
                    SET description = ?1,
                        kind = ?2,
                        configs = ?3
                    WHERE alarm_id = ?4",
            (alarm.get_ref_description(), alarm.get_ref_kind_id(), alarm.get_ref_configs(), id),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Update Alarm: {}", err)))
        ?;

        Ok(())
    }

    fn delete(&mut self, id: String) -> Result<(), VoltaTestError> {
        self.connection.execute(
            "DELETE FROM alarm
                    WHERE alarm_id = ?1",
            (id,),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Delete Alarm: {}", err)))
        ?;

        Ok(())
    }

    fn list(&self, previous_id: Option<String>) -> Result<Vec<AlarmData>, VoltaTestError> {
    
        debug!("Listing from {:#?}", previous_id);
        let query = "
            SELECT alarm_id, description, kind, configs
            FROM alarm
            WHERE alarm_id > ?1
            ORDER BY alarm_id ASC
            LIMIT 20
        ";

        let mut stmt = self.connection.prepare(query)
            .map_err(| err | VoltaTestError::repository_error(err))
            ?;

        let cursor = previous_id.unwrap_or_else(|| "0".to_string());
        let params = (&cursor,);

        let alarms = stmt.query_map(params, |row| {
            let id: i64 = row.get(0)?;
            Ok(AlarmData::new(
                id.to_string(),
                AlarmDescriptor::new(
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                )
            ))
        })
            .map_err(| err | VoltaTestError::repository_error(err))
            ?;

        let mut results = Vec::new();
        for alarm in alarms {
            results.push(
                alarm.map_err(| err | VoltaTestError::repository_error(err))
                ?
            )
        }
        Ok(results)

    }

}
