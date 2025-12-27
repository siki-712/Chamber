//! M014: Unknown decoration name.
//!
//! Checks that decoration names are valid ABC 2.1 standard decorations.

use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};
use chamber_parser::{Decoration, MusicElement, Tune};

use crate::rule::{Category, Rule, RuleMeta};

/// Standard ABC 2.1 decoration names.
const STANDARD_DECORATIONS: &[&str] = &[
    // Dynamics
    "p", "pp", "ppp", "pppp",
    "f", "ff", "fff", "ffff",
    "mp", "mf", "sfz",
    "crescendo", "decrescendo",
    "crescendo(", "crescendo)",
    "decrescendo(", "decrescendo)",
    "<", ">", "<(", "<)", ">(", ">)",
    // Articulation
    "accent", "emphasis",
    "staccato", "staccatissimo",
    "tenuto", "marcato",
    "fermata", "shortfermata", "longfermata",
    "breath",
    // Ornaments
    "trill", "trill(", "trill)",
    "mordent", "pralltriller",
    "lowermordent", "uppermordent",
    "turn", "turnx", "invertedturn",
    "roll",
    "snap", "slide",
    // Bowing / Instrumental
    "upbow", "downbow",
    "open", "plus", "wedge", "thumb",
    "arpeggio",
    // Segno / Coda
    "coda", "segno",
    "D.S.", "D.C.",
    "fine", "dacoda", "dacapo",
    // Phrase marks
    "shortphrase", "mediumphrase", "longphrase",
    // Fingering
    "0", "1", "2", "3", "4", "5",
    // Other
    "invertedfermata",
    "repeatbar", "repeatbar2",
    "upfermata", "downfermata",
];

/// Rule that checks for unknown decoration names.
pub struct UnknownDecoration;

impl RuleMeta for UnknownDecoration {
    const NAME: &'static str = "unknownDecoration";
    const CODE: DiagnosticCode = DiagnosticCode::UnknownDecoration;
    const SEVERITY: Severity = Severity::Error;
    const CATEGORY: Category = Category::Lint;
    const DOCS: &'static str = "Checks that decoration names are valid ABC 2.1 standard decorations.";
}

impl Rule for UnknownDecoration {
    fn run(tune: &Tune, diagnostics: &mut Vec<Diagnostic>) {
        for element in &tune.body.elements {
            check_element(element, diagnostics);
        }
    }
}

fn check_element(element: &MusicElement, diagnostics: &mut Vec<Diagnostic>) {
    match element {
        MusicElement::Note(note) => {
            check_decorations(&note.decorations, diagnostics);
        }
        MusicElement::Rest(rest) => {
            check_decorations(&rest.decorations, diagnostics);
        }
        MusicElement::Chord(chord) => {
            check_decorations(&chord.decorations, diagnostics);
            for note in &chord.notes {
                check_decorations(&note.decorations, diagnostics);
            }
        }
        MusicElement::Slur(slur) => {
            for elem in &slur.elements {
                check_element(elem, diagnostics);
            }
        }
        MusicElement::Tuplet(tuplet) => {
            for note in &tuplet.notes {
                check_decorations(&note.decorations, diagnostics);
            }
        }
        MusicElement::GraceNotes(grace) => {
            for note in &grace.notes {
                check_decorations(&note.decorations, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_decorations(decorations: &[Decoration], diagnostics: &mut Vec<Diagnostic>) {
    for decoration in decorations {
        if !decoration.name.is_empty() && !is_valid_decoration(&decoration.name) {
            let message = if let Some(suggestion) = suggest_decoration(&decoration.name) {
                format!(
                    "unknown decoration '{}', did you mean '{}'?",
                    decoration.name, suggestion
                )
            } else {
                format!("unknown decoration '{}'", decoration.name)
            };

            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownDecoration,
                decoration.range,
                message,
            ));
        }
    }
}

/// Checks if a decoration name is valid (case-insensitive).
fn is_valid_decoration(name: &str) -> bool {
    STANDARD_DECORATIONS
        .iter()
        .any(|&d| d.eq_ignore_ascii_case(name))
}

/// Finds the closest matching decoration name for a typo.
fn suggest_decoration(name: &str) -> Option<&'static str> {
    let name_lower = name.to_lowercase();
    let mut best_match: Option<&'static str> = None;
    let mut best_distance = usize::MAX;

    for &standard in STANDARD_DECORATIONS {
        let distance = levenshtein_distance(&name_lower, &standard.to_lowercase());
        let max_distance = if name.len() <= 4 { 2 } else { name.len() / 2 };

        if distance < best_distance && distance <= max_distance {
            best_distance = distance;
            best_match = Some(standard);
        }
    }

    best_match
}

/// Calculates the Levenshtein distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_decorations() {
        assert!(is_valid_decoration("trill"));
        assert!(is_valid_decoration("TRILL"));
        assert!(is_valid_decoration("Fermata"));
        assert!(is_valid_decoration("accent"));
    }

    #[test]
    fn test_invalid_decorations() {
        assert!(!is_valid_decoration("trillx"));
        assert!(!is_valid_decoration("xyz"));
    }

    #[test]
    fn test_suggestions() {
        assert_eq!(suggest_decoration("trillx"), Some("trill"));
        assert_eq!(suggest_decoration("fermatta"), Some("fermata"));
        assert_eq!(suggest_decoration("akcent"), Some("accent"));
        assert_eq!(suggest_decoration("xyzabc"), None);
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("a", ""), 1);
        assert_eq!(levenshtein_distance("", "a"), 1);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("trill", "trillx"), 1);
    }
}
