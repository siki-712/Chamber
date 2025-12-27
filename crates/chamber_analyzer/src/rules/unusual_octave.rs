//! W001: Unusual octave warning.
//!
//! Warns when notes are in extremely high or low octaves.

use chamber_ast::{MusicElement, Note, Tune};
use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};

use crate::rule::{Category, Rule, RuleMeta};

/// Rule that warns about unusual octaves.
pub struct UnusualOctave;

impl RuleMeta for UnusualOctave {
    const NAME: &'static str = "unusualOctave";
    const CODE: DiagnosticCode = DiagnosticCode::UnusualOctave;
    const SEVERITY: Severity = Severity::Warning;
    const CATEGORY: Category = Category::Lint;
    const DOCS: &'static str = "Warns when notes are in extremely high or low octaves (> 3 or < -2).";
}

impl Rule for UnusualOctave {
    fn run(tune: &Tune, diagnostics: &mut Vec<Diagnostic>) {
        for element in &tune.body.elements {
            check_element(element, diagnostics);
        }
    }
}

fn check_element(element: &MusicElement, diagnostics: &mut Vec<Diagnostic>) {
    match element {
        MusicElement::Note(note) => {
            check_note(note, diagnostics);
        }
        MusicElement::Chord(chord) => {
            for note in &chord.notes {
                check_note(note, diagnostics);
            }
        }
        MusicElement::Slur(slur) => {
            for elem in &slur.elements {
                check_element(elem, diagnostics);
            }
        }
        MusicElement::Tuplet(tuplet) => {
            for note in &tuplet.notes {
                check_note(note, diagnostics);
            }
        }
        MusicElement::GraceNotes(grace) => {
            for note in &grace.notes {
                check_note(note, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_note(note: &Note, diagnostics: &mut Vec<Diagnostic>) {
    if note.octave > 3 || note.octave < -2 {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::UnusualOctave,
            note.range,
            format!(
                "unusual octave {} (notes this {} are rare)",
                note.octave,
                if note.octave > 3 { "high" } else { "low" }
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::RuleExt;
    use chamber_parser::parse;

    #[test]
    fn test_normal_octave() {
        let tune = parse("X:1\nK:C\nCDEF cdef");
        let diagnostics = UnusualOctave::check(&tune);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_high_octave() {
        // c'''' is octave 5
        let tune = parse("X:1\nK:C\nc''''");
        let diagnostics = UnusualOctave::check(&tune);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("high"));
    }

    #[test]
    fn test_low_octave() {
        // C,,, is octave -3
        let tune = parse("X:1\nK:C\nC,,,");
        let diagnostics = UnusualOctave::check(&tune);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("low"));
    }
}
