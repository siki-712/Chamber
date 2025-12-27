//! W002: Suspicious duration warning.
//!
//! Warns when notes have unusually long durations.

use chamber_ast::{Duration, MusicElement, Tune};
use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};

use crate::rule::{Category, Rule, RuleMeta};

/// Rule that warns about suspicious (very long) durations.
pub struct SuspiciousDuration;

impl RuleMeta for SuspiciousDuration {
    const NAME: &'static str = "suspiciousDuration";
    const CODE: DiagnosticCode = DiagnosticCode::SuspiciousDuration;
    const SEVERITY: Severity = Severity::Warning;
    const CATEGORY: Category = Category::Lint;
    const DOCS: &'static str = "Warns when notes have unusually long durations (>= 16 beats).";
}

impl Rule for SuspiciousDuration {
    fn run(tune: &Tune, diagnostics: &mut Vec<Diagnostic>) {
        for element in &tune.body.elements {
            check_element(element, diagnostics);
        }
    }
}

fn check_element(element: &MusicElement, diagnostics: &mut Vec<Diagnostic>) {
    match element {
        MusicElement::Note(note) => {
            check_duration(note.duration.as_ref(), note.range, diagnostics);
        }
        MusicElement::Rest(rest) => {
            check_duration(rest.duration.as_ref(), rest.range, diagnostics);
        }
        MusicElement::Chord(chord) => {
            check_duration(chord.duration.as_ref(), chord.range, diagnostics);
        }
        MusicElement::Slur(slur) => {
            for elem in &slur.elements {
                check_element(elem, diagnostics);
            }
        }
        MusicElement::Tuplet(tuplet) => {
            for note in &tuplet.notes {
                check_duration(note.duration.as_ref(), note.range, diagnostics);
            }
        }
        MusicElement::GraceNotes(grace) => {
            for note in &grace.notes {
                check_duration(note.duration.as_ref(), note.range, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_duration(
    duration: Option<&Duration>,
    range: chamber_text_size::TextRange,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(dur) = duration {
        let effective = dur.numerator as f64 / dur.denominator as f64;
        if effective >= 16.0 {
            diagnostics.push(Diagnostic::warning(
                DiagnosticCode::SuspiciousDuration,
                range,
                format!(
                    "suspicious duration {}/{} (very long note)",
                    dur.numerator, dur.denominator
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::RuleExt;
    use chamber_parser::parse;

    #[test]
    fn test_normal_duration() {
        let tune = parse("X:1\nK:C\nC2 D4 E/2");
        let diagnostics = SuspiciousDuration::check(&tune);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_long_duration() {
        let tune = parse("X:1\nK:C\nC16");
        let diagnostics = SuspiciousDuration::check(&tune);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("suspicious"));
    }

    #[test]
    fn test_very_long_duration() {
        let tune = parse("X:1\nK:C\nC32");
        let diagnostics = SuspiciousDuration::check(&tune);
        assert_eq!(diagnostics.len(), 1);
    }
}
