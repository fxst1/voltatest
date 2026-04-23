use std::fmt::Display;

#[derive(Debug)]
pub enum VoltaTestError {
    /// Related to Alert persistent storage
    RepositoryError(String),

    /// Related to ZMQ and alerting trigger
    AlertingError(String),

    /// Any other kind of error
    Unknown(String)
}

/// Some helpers to convert &str (or any ToString) to String
impl VoltaTestError {
    pub fn repository_error<I: ToString>(reason: I) -> Self {
        VoltaTestError::RepositoryError(reason.to_string())
    }

    pub fn alerting_error<I: ToString>(reason: I) -> Self {
        VoltaTestError::AlertingError(reason.to_string())
    }
}

/// Display trait
impl Display for VoltaTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (source, reason) = match self {
            VoltaTestError::RepositoryError(e) => ("RepositoryError: ", e.to_string()),
            VoltaTestError::AlertingError(e) => ("AlertingError: ", e.to_string()),
            VoltaTestError::Unknown(e) => ("Unknown Error: ", e.to_string())
        };
        f.write_fmt(format_args!("{} {}", source, reason))
    }
}