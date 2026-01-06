//! W003: Bar length mismatch warning.
//!
//! Warns when a bar's total duration does not match the meter.

use chamber_ast::{Duration, HeaderFieldKind, MusicElement, Tune};
use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};
use chamber_text_size::TextRange;

use crate::rule::{Category, Rule, RuleMeta};

/// Rule that warns about bar length mismatches.
pub struct BarLength;

impl RuleMeta for BarLength {
    const NAME: &'static str = "barLength";
    const CODE: DiagnosticCode = DiagnosticCode::BarLengthMismatch;
    const SEVERITY: Severity = Severity::Warning;
    const CATEGORY: Category = Category::Lint;
    const DOCS: &'static str =
        "Warns when a bar's total duration does not match the time signature.";
}

/// Fraction for exact arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fraction {
    num: u32,
    den: u32,
}

impl Fraction {
    fn new(num: u32, den: u32) -> Self {
        Self::reduce(num, den)
    }

    fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    fn add(self, other: Self) -> Self {
        let num = self.num * other.den + other.num * self.den;
        let den = self.den * other.den;
        Self::reduce(num, den)
    }

    fn mul(self, other: Self) -> Self {
        let num = self.num * other.num;
        let den = self.den * other.den;
        Self::reduce(num, den)
    }

    fn reduce(num: u32, den: u32) -> Self {
        if num == 0 {
            return Self { num: 0, den: 1 };
        }
        let g = gcd(num, den);
        Self {
            num: num / g,
            den: den / g,
        }
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Get the time value for a tuplet ratio.
/// ABC standard defaults:
/// - (2 = 2 notes in the time of 3
/// - (3 = 3 notes in the time of 2
/// - (4 = 4 notes in the time of 3
/// - (5+ = n notes in the time of (n-1)
fn tuplet_time(ratio: u32) -> u32 {
    match ratio {
        2 => 3,
        3 | 4 => 2,
        n if n >= 5 => n - 1,
        _ => ratio,
    }
}

impl Rule for BarLength {
    fn run(tune: &Tune, diagnostics: &mut Vec<Diagnostic>) {
        // Get meter from header (default: 4/4)
        let meter = get_meter(tune);

        // Get unit note length from header (default: 1/8)
        let unit_length = get_unit_length(tune);

        // Expected bar length = meter * unit_length factor
        // For M:4/4 with L:1/8, we expect 4/4 of a whole note per bar
        // Each note duration is multiplied by unit_length
        let expected = meter;

        // Track current bar
        let mut bar_start: Option<TextRange> = None;
        let mut bar_total = Fraction::zero();
        let mut bar_end_pos = 0u32;

        for element in &tune.body.elements {
            match element {
                MusicElement::BarLine(barline) => {
                    // Check the completed bar
                    if bar_start.is_some() && bar_total != expected && bar_total != Fraction::zero()
                    {
                        let range = TextRange::new(
                            bar_start.unwrap().start(),
                            barline.range.start(),
                        );
                        diagnostics.push(Diagnostic::warning(
                            DiagnosticCode::BarLengthMismatch,
                            range,
                            format!(
                                "bar has {}/{} beats, expected {}/{}",
                                bar_total.num, bar_total.den, expected.num, expected.den
                            ),
                        ));
                    }
                    // Start new bar
                    bar_start = Some(barline.range);
                    bar_total = Fraction::zero();
                    bar_end_pos = barline.range.end().into();
                }
                MusicElement::Note(note) => {
                    let dur = note_duration(note.duration.as_ref(), unit_length);
                    bar_total = bar_total.add(dur);
                    if bar_start.is_none() {
                        bar_start = Some(note.range);
                    }
                    bar_end_pos = note.range.end().into();
                }
                MusicElement::Rest(rest) => {
                    if !rest.multi_measure {
                        let dur = note_duration(rest.duration.as_ref(), unit_length);
                        bar_total = bar_total.add(dur);
                        if bar_start.is_none() {
                            bar_start = Some(rest.range);
                        }
                        bar_end_pos = rest.range.end().into();
                    }
                }
                MusicElement::Chord(chord) => {
                    let dur = note_duration(chord.duration.as_ref(), unit_length);
                    bar_total = bar_total.add(dur);
                    if bar_start.is_none() {
                        bar_start = Some(chord.range);
                    }
                    bar_end_pos = chord.range.end().into();
                }
                MusicElement::Tuplet(tuplet) => {
                    let time = tuplet_time(tuplet.ratio);
                    let ratio_frac = Fraction::new(time, tuplet.ratio);

                    for note in &tuplet.notes {
                        let note_dur = note_duration(note.duration.as_ref(), unit_length);
                        let effective = note_dur.mul(ratio_frac);
                        bar_total = bar_total.add(effective);
                    }
                    if bar_start.is_none() {
                        bar_start = Some(tuplet.range);
                    }
                    bar_end_pos = tuplet.range.end().into();
                }
                MusicElement::Slur(slur) => {
                    let dur = sum_slur_duration(slur, unit_length);
                    bar_total = bar_total.add(dur);
                    if bar_start.is_none() {
                        bar_start = Some(slur.range);
                    }
                    bar_end_pos = slur.range.end().into();
                }
                MusicElement::GraceNotes(_) | MusicElement::Annotation(_) => {
                    // Grace notes and annotations don't count toward bar length
                }
                MusicElement::InlineField(field) => {
                    // TODO: Handle inline M: and L: changes
                    let _ = field;
                }
                MusicElement::BrokenRhythm(_) | MusicElement::Tie(_) => {
                    // These don't add duration directly
                }
            }
        }

        // Check final bar (may be incomplete - pickup/anacrusis)
        // We don't warn on the last bar if it's shorter (common for pickup measures)
        let _ = bar_end_pos;
    }
}

fn get_meter(tune: &Tune) -> Fraction {
    for field in &tune.header.fields {
        if field.kind == HeaderFieldKind::Meter {
            let value = field.value.trim();
            if value == "C" {
                return Fraction::new(4, 4);
            }
            if value == "C|" {
                return Fraction::new(2, 2);
            }
            if let Some((num, den)) = value.split_once('/') {
                if let (Ok(n), Ok(d)) = (num.trim().parse::<u32>(), den.trim().parse::<u32>()) {
                    if d > 0 {
                        return Fraction::new(n, d);
                    }
                }
            }
        }
    }
    // Default: 4/4
    Fraction::new(4, 4)
}

fn get_unit_length(tune: &Tune) -> Fraction {
    for field in &tune.header.fields {
        if field.kind == HeaderFieldKind::UnitNoteLength {
            let value = field.value.trim();
            if let Some((num, den)) = value.split_once('/') {
                if let (Ok(n), Ok(d)) = (num.trim().parse::<u32>(), den.trim().parse::<u32>()) {
                    if d > 0 {
                        return Fraction::new(n, d);
                    }
                }
            }
        }
    }
    // Default: 1/8
    Fraction::new(1, 8)
}

fn note_duration(duration: Option<&Duration>, unit_length: Fraction) -> Fraction {
    let dur = duration.map_or(Fraction::new(1, 1), |d| Fraction::new(d.numerator, d.denominator));
    dur.mul(unit_length)
}

fn sum_slur_duration(slur: &chamber_ast::Slur, unit_length: Fraction) -> Fraction {
    let mut total = Fraction::zero();
    for element in &slur.elements {
        match element {
            MusicElement::Note(note) => {
                total = total.add(note_duration(note.duration.as_ref(), unit_length));
            }
            MusicElement::Rest(rest) => {
                if !rest.multi_measure {
                    total = total.add(note_duration(rest.duration.as_ref(), unit_length));
                }
            }
            MusicElement::Chord(chord) => {
                total = total.add(note_duration(chord.duration.as_ref(), unit_length));
            }
            MusicElement::Tuplet(tuplet) => {
                let time = tuplet_time(tuplet.ratio);
                let ratio_frac = Fraction::new(time, tuplet.ratio);
                for note in &tuplet.notes {
                    let note_dur = note_duration(note.duration.as_ref(), unit_length);
                    total = total.add(note_dur.mul(ratio_frac));
                }
            }
            MusicElement::Slur(inner) => {
                total = total.add(sum_slur_duration(inner, unit_length));
            }
            MusicElement::GraceNotes(_) => {}
            _ => {}
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::RuleExt;
    use chamber_parser::parse;

    #[test]
    fn test_correct_bar_length() {
        // M:4/4 L:1/8 means 8 eighth notes per bar
        let tune = parse("X:1\nM:4/4\nL:1/8\nK:C\nCDEF GABC|");
        let diagnostics = BarLength::check(&tune);
        assert!(diagnostics.is_empty(), "Expected no warnings for correct bar");
    }

    #[test]
    fn test_bar_too_long() {
        // M:4/4 L:1/8, but 9 eighth notes in bar
        let tune = parse("X:1\nM:4/4\nL:1/8\nK:C\nCDEF GABCD|");
        let diagnostics = BarLength::check(&tune);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("9/8"));
    }

    #[test]
    fn test_bar_too_short() {
        // M:4/4 L:1/8, but only 4 eighth notes in bar
        let tune = parse("X:1\nM:4/4\nL:1/8\nK:C\nCDEF|GABCDEFG|");
        let diagnostics = BarLength::check(&tune);
        assert_eq!(diagnostics.len(), 1);
        // 4 eighth notes = 4/8 = 1/2 (reduced)
        assert!(diagnostics[0].message.contains("1/2"));
    }

    #[test]
    fn test_triplet() {
        // M:2/4 L:1/8 = 4 eighth notes per bar
        // (3CDE = 3 notes in time of 2 = 2 eighth notes
        // FG = 2 eighth notes
        // Total = 4 eighth notes
        let tune = parse("X:1\nM:2/4\nL:1/8\nK:C\n(3CDE FG|");
        let diagnostics = BarLength::check(&tune);
        assert!(diagnostics.is_empty(), "Triplet should be counted correctly");
    }

    #[test]
    fn test_grace_notes_excluded() {
        // Grace notes should not count toward bar length
        let tune = parse("X:1\nM:4/4\nL:1/8\nK:C\n{GAB}CDEF GABC|");
        let diagnostics = BarLength::check(&tune);
        assert!(diagnostics.is_empty(), "Grace notes should be excluded");
    }

    #[test]
    fn test_rest_counted() {
        // M:4/4 L:1/8 = 8 eighth notes per bar
        // CDEF z2 GA = 4 + 2 + 2 = 8
        let tune = parse("X:1\nM:4/4\nL:1/8\nK:C\nCDEF z2GA|");
        let diagnostics = BarLength::check(&tune);
        assert!(diagnostics.is_empty(), "Rests should be counted");
    }
}
