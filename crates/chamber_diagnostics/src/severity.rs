/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Informational message, does not indicate a problem.
    Info,
    /// Warning - the code may work but there is a potential issue.
    Warning,
    /// Error - the code is invalid or cannot be parsed correctly.
    Error,
}

impl Severity {
    /// Returns true if this is an error.
    pub fn is_error(self) -> bool {
        self == Severity::Error
    }

    /// Returns true if this is a warning.
    pub fn is_warning(self) -> bool {
        self == Severity::Warning
    }

    /// Returns a string representation for display.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
