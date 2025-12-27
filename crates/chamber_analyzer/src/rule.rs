//! Rule trait and infrastructure for semantic analysis.

use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};
use chamber_parser::Tune;

/// Category of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Lint rules check for potential errors and bad practices.
    Lint,
    /// Style rules check for formatting and style issues.
    Style,
}

/// Metadata for a rule.
pub trait RuleMeta {
    /// The unique name of the rule (e.g., "unknownDecoration").
    const NAME: &'static str;

    /// The diagnostic code for this rule.
    const CODE: DiagnosticCode;

    /// The default severity.
    const SEVERITY: Severity;

    /// The category of this rule.
    const CATEGORY: Category;

    /// A short description of what this rule checks.
    const DOCS: &'static str;
}

/// A semantic analysis rule.
///
/// Rules are run on the parsed AST to check for semantic issues
/// that cannot be detected during parsing.
pub trait Rule: RuleMeta {
    /// Run the rule on the given tune and collect diagnostics.
    fn run(tune: &Tune, diagnostics: &mut Vec<Diagnostic>);
}

/// Extension trait for running rules.
pub trait RuleExt: Rule {
    /// Run this rule and return the diagnostics.
    fn check(tune: &Tune) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::run(tune, &mut diagnostics);
        diagnostics
    }
}

// Blanket implementation
impl<R: Rule> RuleExt for R {}
