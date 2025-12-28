use chamber_text_size::TextRange;
use serde::{Deserialize, Serialize};

use crate::{DiagnosticCode, Severity};

/// A diagnostic message with location and details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The diagnostic code/category.
    pub code: DiagnosticCode,
    /// The severity level.
    pub severity: Severity,
    /// The source location where this diagnostic applies.
    pub range: TextRange,
    /// The primary message.
    pub message: String,
    /// Optional additional labels pointing to related locations.
    pub labels: Vec<Label>,
    /// Optional notes providing more context.
    pub notes: Vec<String>,
}

/// A secondary label pointing to a related source location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    /// The source location.
    pub range: TextRange,
    /// The label message.
    pub message: String,
}

impl Label {
    /// Creates a new label.
    pub fn new(range: TextRange, message: impl Into<String>) -> Self {
        Self {
            range,
            message: message.into(),
        }
    }
}

impl Diagnostic {
    /// Creates a new diagnostic with the specified severity.
    pub fn new(
        code: DiagnosticCode,
        severity: Severity,
        range: TextRange,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            range,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Creates a new error diagnostic.
    pub fn error(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Error, range, message)
    }

    /// Creates a new warning diagnostic.
    pub fn warning(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Warning, range, message)
    }

    /// Creates a new info diagnostic.
    pub fn info(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Info, range, message)
    }

    /// Creates a diagnostic using the code's default severity and message.
    pub fn from_code(code: DiagnosticCode, range: TextRange) -> Self {
        Self::new(
            code,
            code.default_severity(),
            range,
            code.message_template(),
        )
    }

    /// Adds a secondary label to this diagnostic.
    pub fn with_label(mut self, range: TextRange, message: impl Into<String>) -> Self {
        self.labels.push(Label::new(range, message));
        self
    }

    /// Adds a note to this diagnostic.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Returns true if this diagnostic is an error.
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }

    /// Returns true if this diagnostic is a warning.
    pub fn is_warning(&self) -> bool {
        self.severity.is_warning()
    }
}
