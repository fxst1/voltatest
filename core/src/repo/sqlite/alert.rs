use log::debug;
use rusqlite::{Connection};
use crate::{error::VoltaTestError, types::Alert, repo::{AlertData, AlertRepository}};

pub struct SqliteAlertRepo {
    connection: Connection
}

/// Queries to execute at startup (ensure DB is ready)
const QUERIES: [&str;2] = [
    "
        CREATE TABLE IF NOT EXISTS alert (
                alert_id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT DEFAULT NULL,
                at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
                data BLOB NOT NULL
        );
    ",

    "CREATE INDEX IF NOT EXISTS idx_alert_at ON alert (at);",
];

impl SqliteAlertRepo {

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


impl AlertRepository for SqliteAlertRepo {

    /// Create an alert
    fn create(&mut self, alert: Alert) -> Result<AlertData, VoltaTestError> {
        self.connection.execute(
            "INSERT INTO alert (
                    description,
                    at,
                    data
            ) VALUES (?1, ?2, ?3)",
            (alert.get_ref_description(), alert.get_ref_timestamp(), alert.get_ref_data()),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Create Alert: {}", err)))
        ?;

        Ok(AlertData::new(
            self.connection.last_insert_rowid().to_string(),
            alert
        ))
    }

    /// Read an alert
    fn read(&self, id: String) -> Result<Option<AlertData>, VoltaTestError> {

        let mut stmt = self.connection.prepare("
            SELECT alert_id, description, at, data
            FROM alert
            WHERE alert_id = ?1
            LIMIT 1
        ")
            .map_err(| err | VoltaTestError::repository_error(format!("read {}: {}", id, err)))
        ?;

        // Reading only one item
        let alert_result = stmt.query_one([&id], |row| {
            let id: i64 = row.get(0)?;
            Ok(AlertData::new(
                id.to_string(),
                Alert::new(
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                )
            ))
        });

        match alert_result {
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

    fn update(&mut self, id: String, alert: Alert) -> Result<(), VoltaTestError> {

        self.connection.execute(
            "UPDATE alert
                    SET description = ?1,
                        at = ?2,
                        data = ?3
                    WHERE alert_id = ?4",
            (alert.get_ref_description(), alert.get_ref_timestamp(), alert.get_ref_data(), id),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Update Alert: {}", err)))
        ?;

        Ok(())
    }

    fn delete(&mut self, id: String) -> Result<(), VoltaTestError> {
        self.connection.execute(
            "DELETE FROM alert
                    WHERE alert_id = ?1",
            (id,),
        )
        .map_err(|err | VoltaTestError::repository_error(format!("Delete Alert: {}", err)))
        ?;

        Ok(())
    }

    fn list(&self, previous_id: Option<String>) -> Result<Vec<AlertData>, VoltaTestError> {
    
        debug!("Listing from {:#?}", previous_id);
        let query = "
            SELECT alert_id, description, at, data
            FROM alert
            WHERE alert_id > ?1
            ORDER BY at ASC
            LIMIT 20
        ";

        let mut stmt = self.connection.prepare(query)
            .map_err(| err | VoltaTestError::repository_error(err))
            ?;

        let cursor = previous_id.unwrap_or_else(|| "0".to_string());
        let params = (&cursor,);

        let alerts = stmt.query_map(params, |row| {
            let id: i64 = row.get(0)?;
            Ok(AlertData::new(
                id.to_string(),
                Alert::new(
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                )
            ))
        })
            .map_err(| err | VoltaTestError::repository_error(err))
            ?;

        let mut results = Vec::new();
        for alert in alerts {
            results.push(
                alert.map_err(| err | VoltaTestError::repository_error(err))
                ?
            )
        }
        Ok(results)

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use crate::repo::AlertRepository;

    fn ts(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn make_alert(description: &str, timestamp: NaiveDateTime) -> Alert {
        Alert::new(Some(description.to_string()), timestamp, vec![1, 2, 3])
    }

    #[test]
    fn create_and_read() {
        let mut repo = SqliteAlertRepo::from_memory().unwrap();
        let alert = make_alert("hello", ts("2024-01-01 10:00:00"));
        let data = repo.create(alert.clone()).unwrap();

        let result = repo.read(data.get_key()).unwrap();
        assert!(result.is_some());
        let stored = result.unwrap();
        assert_eq!(stored.get_value().get_ref_description(), alert.get_ref_description());
        assert_eq!(stored.get_value().get_ref_timestamp(), alert.get_ref_timestamp());
        assert_eq!(stored.get_value().get_ref_data(), alert.get_ref_data());
    }

    #[test]
    fn read_nonexistent_returns_none() {
        let repo = SqliteAlertRepo::from_memory().unwrap();
        let result = repo.read("999".to_string()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn update_modifies_alert() {
        let mut repo = SqliteAlertRepo::from_memory().unwrap();
        let data = repo.create(make_alert("original", ts("2024-01-01 10:00:00"))).unwrap();
        let id = data.get_key();

        let updated = make_alert("updated", ts("2024-06-01 12:00:00"));
        repo.update(id.clone(), updated.clone()).unwrap();

        let stored = repo.read(id).unwrap().unwrap();
        assert_eq!(stored.get_value().get_ref_description(), updated.get_ref_description());
        assert_eq!(stored.get_value().get_ref_timestamp(), updated.get_ref_timestamp());
    }

    #[test]
    fn delete_removes_alert() {
        let mut repo = SqliteAlertRepo::from_memory().unwrap();
        let data = repo.create(make_alert("to_delete", ts("2024-01-01 10:00:00"))).unwrap();
        let id = data.get_key();

        repo.delete(id.clone()).unwrap();
        assert!(repo.read(id).unwrap().is_none());
    }

    #[test]
    fn list_returns_all_without_cursor() {
        let mut repo = SqliteAlertRepo::from_memory().unwrap();
        let t = ts("2024-01-01 10:00:00");
        repo.create(make_alert("a", t)).unwrap();
        repo.create(make_alert("b", t)).unwrap();
        repo.create(make_alert("c", t)).unwrap();

        let results = repo.list( None).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn list_pagination_with_cursor() {
        let mut repo = SqliteAlertRepo::from_memory().unwrap();
        let t = ts("2024-01-01 10:00:00");
        repo.create(make_alert("a", t)).unwrap();
        let second = repo.create(make_alert("b", t)).unwrap();
        repo.create(make_alert("c", t)).unwrap();

        let results = repo.list(Some(second.get_key())).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_value().get_ref_description(), &Some("c".to_string()));
    }

    #[test]
    fn list_empty_repo() {
        let repo = SqliteAlertRepo::from_memory().unwrap();
        let results = repo.list(None).unwrap();
        assert!(results.is_empty());
    }
}
