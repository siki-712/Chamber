//! Semantic analyzer for ABC notation.
//!
//! This crate provides semantic analysis (linting) for ABC notation ASTs.
//! It is separate from the parser, which handles syntax only.

pub mod rules;

use chamber_diagnostics::Diagnostic;
use chamber_parser::Tune;

/// Analyzes a tune and returns diagnostics.
pub fn analyze(tune: &Tune) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Run all rules
    rules::unknown_decoration::check(tune, &mut diagnostics);

    diagnostics
}

/// Result of analysis containing all diagnostics.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisResult {
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_warning())
    }
}
