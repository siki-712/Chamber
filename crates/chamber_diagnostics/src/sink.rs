use crate::{Diagnostic, Severity};

/// Trait for collecting diagnostics during parsing.
pub trait DiagnosticSink {
    /// Reports a diagnostic.
    fn report(&mut self, diagnostic: Diagnostic);

    /// Returns true if any errors have been reported.
    fn has_errors(&self) -> bool;

    /// Returns true if any warnings have been reported.
    fn has_warnings(&self) -> bool;
}

/// Simple vector-based diagnostic collector.
#[derive(Debug, Default, Clone)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
}

impl DiagnosticBag {
    /// Creates a new empty diagnostic bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all collected diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes the bag and returns all diagnostics.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Returns the number of errors.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Returns the total number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns true if no diagnostics have been collected.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Clears all collected diagnostics.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }

    /// Returns an iterator over the diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Returns only the error diagnostics.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error())
    }

    /// Returns only the warning diagnostics.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_warning())
    }
}

impl DiagnosticSink for DiagnosticBag {
    fn report(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            Severity::Info => {}
        }
        self.diagnostics.push(diagnostic);
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
}

impl<'a> IntoIterator for &'a DiagnosticBag {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

impl IntoIterator for DiagnosticBag {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}
