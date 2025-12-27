use chamber_parser::*;

#[test]
fn test_empty_tune() {
    let tune = parse("");
    assert!(tune.header.fields.is_empty());
    assert!(tune.body.elements.is_empty());
}

#[test]
fn test_header_fields() {
    let tune = parse("X:1\nT:Test Tune\nM:4/4\nK:C\n");

    assert_eq!(tune.header.fields.len(), 4);

    assert_eq!(tune.header.fields[0].kind, HeaderFieldKind::ReferenceNumber);
    assert_eq!(tune.header.fields[0].value, "1");

    assert_eq!(tune.header.fields[1].kind, HeaderFieldKind::Title);
    assert_eq!(tune.header.fields[1].value, "Test Tune");

    assert_eq!(tune.header.fields[2].kind, HeaderFieldKind::Meter);
    assert_eq!(tune.header.fields[2].value, "4/4");

    assert_eq!(tune.header.fields[3].kind, HeaderFieldKind::Key);
    assert_eq!(tune.header.fields[3].value, "C");
}

#[test]
fn test_simple_notes() {
    let tune = parse("X:1\nK:C\nCDEF");

    assert_eq!(tune.body.elements.len(), 4);

    for (i, pitch) in [Pitch::C, Pitch::D, Pitch::E, Pitch::F].iter().enumerate() {
        match &tune.body.elements[i] {
            MusicElement::Note(note) => {
                assert_eq!(note.pitch, *pitch);
                assert_eq!(note.octave, 0);
                assert!(note.accidental.is_none());
            }
            _ => panic!("Expected Note"),
        }
    }
}

#[test]
fn test_lowercase_notes() {
    let tune = parse("X:1\nK:C\ncdef");

    for (i, pitch) in [Pitch::C, Pitch::D, Pitch::E, Pitch::F].iter().enumerate() {
        match &tune.body.elements[i] {
            MusicElement::Note(note) => {
                assert_eq!(note.pitch, *pitch);
                assert_eq!(note.octave, 1); // Lowercase = octave up
            }
            _ => panic!("Expected Note"),
        }
    }
}

#[test]
fn test_octave_modifiers() {
    let tune = parse("X:1\nK:C\nC'C''c,c,,");

    let expected = [(Pitch::C, 1), (Pitch::C, 2), (Pitch::C, 0), (Pitch::C, -1)];

    for (i, (pitch, octave)) in expected.iter().enumerate() {
        match &tune.body.elements[i] {
            MusicElement::Note(note) => {
                assert_eq!(note.pitch, *pitch);
                assert_eq!(note.octave, *octave);
            }
            _ => panic!("Expected Note at index {}", i),
        }
    }
}

#[test]
fn test_accidentals() {
    let tune = parse("X:1\nK:C\n^C_D=E^^F__G");

    let expected = [
        (Pitch::C, Some(Accidental::Sharp)),
        (Pitch::D, Some(Accidental::Flat)),
        (Pitch::E, Some(Accidental::Natural)),
        (Pitch::F, Some(Accidental::DoubleSharp)),
        (Pitch::G, Some(Accidental::DoubleFlat)),
    ];

    for (i, (pitch, accidental)) in expected.iter().enumerate() {
        match &tune.body.elements[i] {
            MusicElement::Note(note) => {
                assert_eq!(note.pitch, *pitch);
                assert_eq!(note.accidental, *accidental);
            }
            _ => panic!("Expected Note at index {}", i),
        }
    }
}

#[test]
fn test_note_duration() {
    let tune = parse("X:1\nK:C\nC2D/2E3/4");

    match &tune.body.elements[0] {
        MusicElement::Note(note) => {
            assert_eq!(note.pitch, Pitch::C);
            assert_eq!(note.duration, Some(Duration::new(2, 1)));
        }
        _ => panic!("Expected Note"),
    }

    match &tune.body.elements[1] {
        MusicElement::Note(note) => {
            assert_eq!(note.pitch, Pitch::D);
            assert_eq!(note.duration, Some(Duration::new(1, 2)));
        }
        _ => panic!("Expected Note"),
    }

    match &tune.body.elements[2] {
        MusicElement::Note(note) => {
            assert_eq!(note.pitch, Pitch::E);
            assert_eq!(note.duration, Some(Duration::new(3, 4)));
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn test_rest() {
    let tune = parse("X:1\nK:C\nz2Z4");

    match &tune.body.elements[0] {
        MusicElement::Rest(rest) => {
            assert!(!rest.multi_measure);
            assert_eq!(rest.duration, Some(Duration::new(2, 1)));
        }
        _ => panic!("Expected Rest"),
    }

    match &tune.body.elements[1] {
        MusicElement::Rest(rest) => {
            assert!(rest.multi_measure);
            assert_eq!(rest.duration, Some(Duration::new(4, 1)));
        }
        _ => panic!("Expected Rest"),
    }
}

#[test]
fn test_bar_lines() {
    let tune = parse("X:1\nK:C\n|C|D||E|:F:|");

    let mut bar_count = 0;
    let expected_bars = [
        BarLineKind::Single,
        BarLineKind::Single,
        BarLineKind::Double,
        BarLineKind::RepeatStart,
        BarLineKind::RepeatEnd,
    ];

    for element in &tune.body.elements {
        if let MusicElement::BarLine(bar) = element {
            assert_eq!(bar.kind, expected_bars[bar_count]);
            bar_count += 1;
        }
    }

    assert_eq!(bar_count, 5);
}

#[test]
fn test_chord() {
    let tune = parse("X:1\nK:C\n[CEG]2");

    match &tune.body.elements[0] {
        MusicElement::Chord(chord) => {
            assert_eq!(chord.notes.len(), 3);
            assert_eq!(chord.notes[0].pitch, Pitch::C);
            assert_eq!(chord.notes[1].pitch, Pitch::E);
            assert_eq!(chord.notes[2].pitch, Pitch::G);
            assert_eq!(chord.duration, Some(Duration::new(2, 1)));
        }
        _ => panic!("Expected Chord"),
    }
}

#[test]
fn test_tuplet() {
    let tune = parse("X:1\nK:C\n(3CDE");

    match &tune.body.elements[0] {
        MusicElement::Tuplet(tuplet) => {
            assert_eq!(tuplet.ratio, 3);
            assert_eq!(tuplet.notes.len(), 3);
            assert_eq!(tuplet.notes[0].pitch, Pitch::C);
            assert_eq!(tuplet.notes[1].pitch, Pitch::D);
            assert_eq!(tuplet.notes[2].pitch, Pitch::E);
        }
        _ => panic!("Expected Tuplet"),
    }
}

#[test]
fn test_slur() {
    let tune = parse("X:1\nK:C\n(CDE)");

    match &tune.body.elements[0] {
        MusicElement::Slur(slur) => {
            assert_eq!(slur.elements.len(), 3);
        }
        _ => panic!("Expected Slur"),
    }
}

#[test]
fn test_grace_notes() {
    let tune = parse("X:1\nK:C\n{g}C");

    match &tune.body.elements[0] {
        MusicElement::GraceNotes(grace) => {
            assert_eq!(grace.notes.len(), 1);
            assert_eq!(grace.notes[0].pitch, Pitch::G);
        }
        _ => panic!("Expected GraceNotes"),
    }

    match &tune.body.elements[1] {
        MusicElement::Note(note) => {
            assert_eq!(note.pitch, Pitch::C);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn test_tie() {
    let tune = parse("X:1\nK:C\nC-C");

    match &tune.body.elements[0] {
        MusicElement::Note(note) => assert_eq!(note.pitch, Pitch::C),
        _ => panic!("Expected Note"),
    }

    match &tune.body.elements[1] {
        MusicElement::Tie(_) => {}
        _ => panic!("Expected Tie"),
    }

    match &tune.body.elements[2] {
        MusicElement::Note(note) => assert_eq!(note.pitch, Pitch::C),
        _ => panic!("Expected Note"),
    }
}

#[test]
fn test_broken_rhythm() {
    let tune = parse("X:1\nK:C\nC>D");

    match &tune.body.elements[0] {
        MusicElement::Note(note) => assert_eq!(note.pitch, Pitch::C),
        _ => panic!("Expected Note"),
    }

    match &tune.body.elements[1] {
        MusicElement::BrokenRhythm(br) => {
            assert!(br.dotted_first);
            assert_eq!(br.count, 1);
        }
        _ => panic!("Expected BrokenRhythm"),
    }

    match &tune.body.elements[2] {
        MusicElement::Note(note) => assert_eq!(note.pitch, Pitch::D),
        _ => panic!("Expected Note"),
    }
}

#[test]
fn test_complete_tune() {
    let source = r#"X:1
T:Twinkle Twinkle
M:4/4
L:1/4
K:C
CCGG|AAG2|FFEE|DDC2|
"#;

    let tune = parse(source);

    assert_eq!(tune.header.fields.len(), 5);
    assert_eq!(tune.header.fields[0].kind, HeaderFieldKind::ReferenceNumber);
    assert_eq!(tune.header.fields[1].kind, HeaderFieldKind::Title);
    assert_eq!(tune.header.fields[1].value, "Twinkle Twinkle");

    // Count notes and bar lines
    let mut note_count = 0;
    let mut bar_count = 0;

    for element in &tune.body.elements {
        match element {
            MusicElement::Note(_) => note_count += 1,
            MusicElement::BarLine(_) => bar_count += 1,
            _ => {}
        }
    }

    // CCGG|AAG2|FFEE|DDC2| = 14 notes (G2 and C2 are single notes with duration)
    assert_eq!(note_count, 14);
    assert_eq!(bar_count, 4);
}

// ============================================
// Diagnostic tests
// ============================================

use chamber_diagnostics::DiagnosticCode;

#[test]
fn test_parse_with_diagnostics_no_errors() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nCDEF");

    assert!(!result.has_errors());
    assert!(!result.has_warnings());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_unclosed_chord() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\n[CEG");

    assert!(result.has_errors());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, DiagnosticCode::UnclosedChord);

    // The chord should still be parsed with partial content
    match &result.tune.body.elements[0] {
        MusicElement::Chord(chord) => {
            assert_eq!(chord.notes.len(), 3);
        }
        _ => panic!("Expected Chord"),
    }
}

#[test]
fn test_unclosed_slur() {
    let result = parse_with_diagnostics("X:1\nK:C\n(CDE");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedSlur));
}

#[test]
fn test_unclosed_grace_notes() {
    let result = parse_with_diagnostics("X:1\nK:C\n{g");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedGraceNotes));
}

#[test]
fn test_unexpected_closing_bracket() {
    let result = parse_with_diagnostics("X:1\nK:C\nCDE]F");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedClosingBracket));
}

#[test]
fn test_unexpected_closing_paren() {
    let result = parse_with_diagnostics("X:1\nK:C\nCDE)F");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedClosingParen));
}

#[test]
fn test_unexpected_closing_brace() {
    let result = parse_with_diagnostics("X:1\nK:C\nCDE}F");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedClosingBrace));
}

#[test]
fn test_unexpected_character() {
    let result = parse_with_diagnostics("X:1\nK:C\nC@D#E");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedCharacter));
}

#[test]
fn test_empty_chord_warning() {
    let result = parse_with_diagnostics("X:1\nK:C\n[]");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyChord));
}

#[test]
fn test_empty_tuplet_warning() {
    let result = parse_with_diagnostics("X:1\nK:C\n(3|");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyTuplet));
}

#[test]
fn test_tuplet_note_mismatch_warning() {
    let result = parse_with_diagnostics("X:1\nK:C\n(3CD|");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::TupletNoteMismatch));
}

#[test]
fn test_error_recovery_at_barline() {
    // Parser should recover at bar line when chord is unclosed
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\n|DEF");

    assert!(result.has_errors());

    // Should have an unclosed chord error
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));

    // Should have parsed the chord with the notes before the newline
    let has_chord = result
        .tune
        .body
        .elements
        .iter()
        .any(|e| matches!(e, MusicElement::Chord(_)));
    assert!(has_chord, "Should have parsed a chord");
}

#[test]
fn test_multiple_unexpected_chars() {
    // Multiple unexpected characters should generate multiple diagnostics
    let result = parse_with_diagnostics("X:1\nK:C\nC@D#E");

    // Should have at least 2 unexpected character errors
    let unexpected_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnexpectedCharacter)
        .count();

    assert!(
        unexpected_count >= 2,
        "Expected at least 2 unexpected character errors, got {}",
        unexpected_count
    );
}

// ============================================
// W001: Unusual octave
// ============================================

#[test]
fn test_unusual_octave_high() {
    // C'''' = octave 4 (very high)
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC''''");

    assert!(result.has_warnings());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnusualOctave),
        "Expected UnusualOctave warning, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_unusual_octave_low() {
    // C,,, = octave -3 (very low)
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC,,,");

    assert!(result.has_warnings());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnusualOctave),
        "Expected UnusualOctave warning, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_normal_octave_no_warning() {
    // Normal octaves should not warn
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC,Cc'c''");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnusualOctave),
        "Unexpected UnusualOctave warning: {:?}",
        result.diagnostics
    );
}

// ============================================
// W002: Suspicious duration
// ============================================

#[test]
fn test_suspicious_duration_very_long() {
    // C64 = 64 beats (very long)
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC64");

    assert!(result.has_warnings());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::SuspiciousDuration),
        "Expected SuspiciousDuration warning, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_suspicious_duration_large_fraction() {
    // C32/1 = 32 beats
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC32/1");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::SuspiciousDuration),
        "Expected SuspiciousDuration warning, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_normal_duration_no_warning() {
    // Normal durations should not warn
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC2D4E/2F8");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::SuspiciousDuration),
        "Unexpected SuspiciousDuration warning: {:?}",
        result.diagnostics
    );
}

// ============================================
// M009: Invalid duration
// ============================================

#[test]
fn test_invalid_duration_zero_denominator() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nC/0");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidDuration),
        "Expected InvalidDuration error, got: {:?}",
        result.diagnostics
    );
}

// ============================================
// Inline fields
// ============================================

#[test]
fn test_inline_field_meter() {
    let tune = parse("X:1\nK:C\nCDEF [M:3/4] GAB|");

    // Should have: C, D, E, F, InlineField, G, A, B, BarLine
    assert!(tune.body.elements.len() >= 5);

    match &tune.body.elements[4] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'M');
            assert_eq!(f.value, "3/4");
        }
        other => panic!("Expected InlineField, got {:?}", other),
    }
}

#[test]
fn test_inline_field_key() {
    let tune = parse("X:1\nK:C\nCDEF [K:G] GAB|");

    match &tune.body.elements[4] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'K');
            assert_eq!(f.value, "G");
        }
        other => panic!("Expected InlineField, got {:?}", other),
    }
}

#[test]
fn test_inline_field_tempo() {
    let tune = parse("X:1\nK:C\n[Q:120] CDEF|");

    match &tune.body.elements[0] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'Q');
            assert_eq!(f.value, "120");
        }
        other => panic!("Expected InlineField, got {:?}", other),
    }
}

#[test]
fn test_inline_field_unit_note_length() {
    let tune = parse("X:1\nK:C\n[L:1/16] CDEF|");

    match &tune.body.elements[0] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'L');
            assert_eq!(f.value, "1/16");
        }
        other => panic!("Expected InlineField, got {:?}", other),
    }
}

#[test]
fn test_chord_still_works() {
    let tune = parse("X:1\nK:C\n[CEG]|");

    match &tune.body.elements[0] {
        MusicElement::Chord(chord) => {
            assert_eq!(chord.notes.len(), 3);
            assert_eq!(chord.notes[0].pitch, Pitch::C);
            assert_eq!(chord.notes[1].pitch, Pitch::E);
            assert_eq!(chord.notes[2].pitch, Pitch::G);
        }
        other => panic!("Expected Chord, got {:?}", other),
    }
}

#[test]
fn test_inline_field_vs_chord() {
    // Both inline fields and chords in same tune
    let tune = parse("X:1\nK:C\n[CEG] [M:3/4] [FAC]|");

    // Element 0: Chord [CEG]
    match &tune.body.elements[0] {
        MusicElement::Chord(chord) => {
            assert_eq!(chord.notes.len(), 3);
        }
        other => panic!("Expected Chord at index 0, got {:?}", other),
    }

    // Element 1: InlineField [M:3/4]
    match &tune.body.elements[1] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'M');
            assert_eq!(f.value, "3/4");
        }
        other => panic!("Expected InlineField at index 1, got {:?}", other),
    }

    // Element 2: Chord [FAC]
    match &tune.body.elements[2] {
        MusicElement::Chord(chord) => {
            assert_eq!(chord.notes.len(), 3);
        }
        other => panic!("Expected Chord at index 2, got {:?}", other),
    }
}

#[test]
fn test_unclosed_inline_field() {
    let result = parse_with_diagnostics("X:1\nK:C\n[M:3/4");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnclosedInlineField),
        "Expected UnclosedInlineField error, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_inline_field_with_complex_value() {
    // Inline field with a more complex value
    let tune = parse("X:1\nK:C\n[Q:1/4=120]|");

    match &tune.body.elements[0] {
        MusicElement::InlineField(f) => {
            assert_eq!(f.label, 'Q');
            assert_eq!(f.value, "1/4=120");
        }
        other => panic!("Expected InlineField, got {:?}", other),
    }
}
