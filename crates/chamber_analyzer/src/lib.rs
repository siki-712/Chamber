//! Semantic analyzer for ABC notation.
//!
//! This crate provides semantic analysis (linting) for ABC notation ASTs.
//! It is separate from the parser, which handles syntax only.
//!
//! # Architecture
//!
//! The analyzer follows a rule-based architecture inspired by Biome:
//!
//! - Each rule implements the `Rule` trait
//! - Rules are registered in the `Analyzer`
//! - The analyzer runs all rules and collects diagnostics
//!
//! # Example
//!
//! ```
//! use chamber_analyzer::Analyzer;
//! use chamber_parser::parse;
//!
//! let tune = parse("X:1\nK:C\n!trillx!C");
//! let result = Analyzer::new().analyze(&tune);
//!
//! for diagnostic in result.diagnostics {
//!     println!("{}: {}", diagnostic.code, diagnostic.message);
//! }
//! ```

mod rule;
pub mod rules;

use chamber_ast::Tune;
use chamber_diagnostics::Diagnostic;

pub use rule::{Category, Rule, RuleExt, RuleMeta};
pub use rules::{SuspiciousDuration, UnknownDecoration, UnusualOctave};

/// Result of semantic analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Diagnostics collected during analysis.
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisResult {
    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_warning())
    }
}

/// The semantic analyzer.
///
/// Runs all registered rules on an AST and collects diagnostics.
#[derive(Debug, Default)]
pub struct Analyzer {
    /// Whether to run lint rules.
    pub lint: bool,
    /// Whether to run style rules.
    pub style: bool,
}

impl Analyzer {
    /// Creates a new analyzer with all rules enabled.
    pub fn new() -> Self {
        Self {
            lint: true,
            style: true,
        }
    }

    /// Disables lint rules.
    pub fn without_lint(mut self) -> Self {
        self.lint = false;
        self
    }

    /// Disables style rules.
    pub fn without_style(mut self) -> Self {
        self.style = false;
        self
    }

    /// Analyzes a tune and returns the result.
    pub fn analyze(&self, tune: &Tune) -> AnalysisResult {
        let mut diagnostics = Vec::new();

        if self.lint {
            // Run lint rules
            UnknownDecoration::run(tune, &mut diagnostics);
            UnusualOctave::run(tune, &mut diagnostics);
            SuspiciousDuration::run(tune, &mut diagnostics);
        }

        // Sort diagnostics by position for consistent output
        diagnostics.sort_by_key(|d| d.range.start());

        AnalysisResult { diagnostics }
    }
}

/// Convenience function to analyze a tune with default settings.
pub fn analyze(tune: &Tune) -> Vec<Diagnostic> {
    Analyzer::new().analyze(tune).diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_parser::parse;

    #[test]
    fn test_analyzer_runs_all_rules() {
        let tune = parse("X:1\nK:C\n!trillx!C''''16");
        let result = Analyzer::new().analyze(&tune);

        // Should have errors from multiple rules
        assert!(result.has_errors()); // M014
        assert!(result.has_warnings()); // W001, W002
    }

    #[test]
    fn test_analyzer_can_disable_lint() {
        let tune = parse("X:1\nK:C\n!trillx!C");
        let result = Analyzer::new().without_lint().analyze(&tune);

        // No diagnostics when lint is disabled
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_diagnostics_sorted_by_position() {
        let tune = parse("X:1\nK:C\n!b!C !a!D");
        let result = Analyzer::new().analyze(&tune);

        // Diagnostics should be sorted by position
        for i in 1..result.diagnostics.len() {
            assert!(result.diagnostics[i - 1].range.start() <= result.diagnostics[i].range.start());
        }
    }
}
