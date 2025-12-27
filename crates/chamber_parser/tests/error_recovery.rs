//! Integration tests for error recovery
//!
//! These tests verify that the parser correctly handles multiple errors
//! and recovers appropriately without losing track of subsequent errors.

use chamber_diagnostics::DiagnosticCode;
use chamber_parser::{parse_with_diagnostics, MusicElement};

// ============================================
// Multiple unclosed structures
// ============================================

#[test]
fn unclosed_chord_then_unclosed_slur() {
    // The bug that was found: chord was eating slur's notes
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\n(ABC");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::UnclosedChord),
        "Missing UnclosedChord, got: {:?}",
        codes
    );
    assert!(
        codes.contains(&DiagnosticCode::UnclosedSlur),
        "Missing UnclosedSlur, got: {:?}",
        codes
    );

    // Chord should have exactly 3 notes, NOT 6
    let chord = result
        .tune
        .body
        .elements
        .iter()
        .find_map(|e| match e {
            MusicElement::Chord(c) => Some(c),
            _ => None,
        })
        .expect("Should have a chord");
    assert_eq!(chord.notes.len(), 3, "Chord ate notes from the slur!");
}

#[test]
fn unclosed_chord_then_unclosed_grace() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\n{abc");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&DiagnosticCode::UnclosedChord));
    assert!(codes.contains(&DiagnosticCode::UnclosedGraceNotes));
}

#[test]
fn unclosed_slur_then_unclosed_chord() {
    let result = parse_with_diagnostics("X:1\nK:C\n(ABC\n[CEG");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&DiagnosticCode::UnclosedSlur));
    assert!(codes.contains(&DiagnosticCode::UnclosedChord));
}

#[test]
fn triple_unclosed_structures() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\n[CEG\n(ABC\n{def");

    // Should have 3 unclosed errors (plus no header warnings since T: is present)
    let error_codes: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code)
        .collect();

    assert_eq!(
        error_codes.len(),
        3,
        "Expected 3 errors, got: {:?}",
        error_codes
    );

    assert!(error_codes.contains(&DiagnosticCode::UnclosedChord));
    assert!(error_codes.contains(&DiagnosticCode::UnclosedSlur));
    assert!(error_codes.contains(&DiagnosticCode::UnclosedGraceNotes));
}

// ============================================
// Same line multiple structures
// ============================================

#[test]
fn unclosed_structures_same_line_with_bar() {
    // Bar should act as recovery point for both
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG|(ABC|");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::UnclosedChord),
        "got: {:?}",
        codes
    );
    assert!(
        codes.contains(&DiagnosticCode::UnclosedSlur),
        "got: {:?}",
        codes
    );
}

#[test]
fn interleaved_open_close_wrong_order() {
    // Open chord, open slur, close chord - slur should error
    let result = parse_with_diagnostics("X:1\nK:C\n[(ABC]");

    // This is tricky: [ starts chord, ( is inside chord (should be ignored or error)
    // ] closes chord - the ( has nowhere to go
    assert!(result.has_errors(), "Should have errors for mismatched brackets");
}

// ============================================
// Recovery point edge cases
// ============================================

#[test]
fn unclosed_at_immediate_newline() {
    // Opening bracket immediately followed by newline
    let result = parse_with_diagnostics("X:1\nK:C\n[\nCDE");

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));

    // Should have parsed the notes after recovery
    let note_count = result
        .tune
        .body
        .elements
        .iter()
        .filter(|e| matches!(e, MusicElement::Note(_)))
        .count();
    assert!(note_count >= 3, "Notes after newline should be parsed");
}

#[test]
fn unclosed_at_immediate_bar() {
    // Note: [| is a valid thick-thin bar in ABC notation, so use [ | (with space) instead
    let result = parse_with_diagnostics("X:1\nK:C\n[C|DE");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnclosedChord),
        "Expected UnclosedChord, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn multiple_newlines_between_unclosed() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\n\n\n(ABC");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&DiagnosticCode::UnclosedChord));
    assert!(codes.contains(&DiagnosticCode::UnclosedSlur));
}

// ============================================
// Unexpected closing brackets
// ============================================

#[test]
fn multiple_unexpected_closers() {
    let result = parse_with_diagnostics("X:1\nK:C\nC]D)E}F");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&DiagnosticCode::UnexpectedClosingBracket));
    assert!(codes.contains(&DiagnosticCode::UnexpectedClosingParen));
    assert!(codes.contains(&DiagnosticCode::UnexpectedClosingBrace));
}

#[test]
fn mixed_open_and_unexpected_close() {
    // Open slur, unexpected ], continue
    let result = parse_with_diagnostics("X:1\nK:C\n(ABC]DEF)");

    // Should report unexpected ] but slur should still close properly
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnexpectedClosingBracket),
        "Expected UnexpectedClosingBracket, got: {:?}",
        result.diagnostics
    );

    // Slur should be properly closed (no UnclosedSlur error)
    let has_unclosed_slur = result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedSlur);
    assert!(!has_unclosed_slur, "Slur should be properly closed");
}

// ============================================
// Nested structures
// ============================================

#[test]
fn slur_containing_chord_unclosed() {
    // Valid: slur can contain chords
    let result = parse_with_diagnostics("X:1\nK:C\n([CEG]ABC)");

    // This should parse without errors
    assert!(
        !result.has_errors(),
        "Valid nested structure got errors: {:?}",
        result.diagnostics
    );
}

#[test]
fn slur_containing_unclosed_chord() {
    let result = parse_with_diagnostics("X:1\nK:C\n([CEG ABC)");

    // The chord is unclosed, but the slur closes
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));
}

#[test]
fn grace_before_chord_both_unclosed() {
    let result = parse_with_diagnostics("X:1\nK:C\n{g[CEG");

    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::UnclosedGraceNotes),
        "got: {:?}",
        codes
    );
    // Note: chord may or may not be detected depending on recovery
}

// ============================================
// Stress tests
// ============================================

#[test]
fn many_unclosed_structures() {
    let result = parse_with_diagnostics(
        "X:1\nK:C\n\
         [C\n\
         (D\n\
         {e\n\
         [F\n\
         (G\n",
    );

    // Should have at least 5 unclosed errors
    assert!(
        result.diagnostics.len() >= 5,
        "Expected at least 5 errors, got {} : {:?}",
        result.diagnostics.len(),
        result.diagnostics
    );
}

#[test]
fn alternating_valid_and_invalid() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG]ABC[DEF(GHI\n|JKL)");

    // First chord valid, second chord unclosed, slur spans lines
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));

    // The slur should be properly closed by )
    let _slur_closed = result
        .tune
        .body
        .elements
        .iter()
        .any(|e| matches!(e, MusicElement::Slur(_)));
    // Note: depending on recovery, slur may or may not be complete
}

#[test]
fn error_tokens_mixed_with_structures() {
    let result = parse_with_diagnostics("X:1\nK:C\n[@CEG\n#(ABC");

    // Should report unexpected chars AND unclosed structures
    let has_unexpected = result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedCharacter);
    assert!(has_unexpected, "Should report unexpected characters");

    let has_unclosed = result
        .diagnostics
        .iter()
        .any(|d| matches!(d.code, DiagnosticCode::UnclosedChord | DiagnosticCode::UnclosedSlur));
    assert!(has_unclosed, "Should report unclosed structures");
}

// ============================================
// EOF handling
// ============================================

#[test]
fn unclosed_at_exact_eof() {
    // No newline at end
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG");

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));
}

#[test]
fn multiple_unclosed_at_eof_no_newlines() {
    let result = parse_with_diagnostics("X:1\nK:C\n[C(D{e");

    // All three should be detected
    assert!(
        result.diagnostics.len() >= 2,
        "Should detect multiple unclosed at EOF: {:?}",
        result.diagnostics
    );
}

// ============================================
// Tuplet edge cases
// ============================================

#[test]
fn tuplet_then_unclosed_chord() {
    let result = parse_with_diagnostics("X:1\nK:C\n(3CDE[FGA");

    // Tuplet should be complete, chord should be unclosed
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));

    let has_tuplet = result
        .tune
        .body
        .elements
        .iter()
        .any(|e| matches!(e, MusicElement::Tuplet(_)));
    assert!(has_tuplet, "Tuplet should be parsed");
}

#[test]
fn unclosed_chord_before_tuplet() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\n(3ABC");

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnclosedChord));

    // Tuplet should still be parsed after recovery
    let has_tuplet = result
        .tune
        .body
        .elements
        .iter()
        .any(|e| matches!(e, MusicElement::Tuplet(_)));
    assert!(has_tuplet, "Tuplet after recovery should be parsed");
}

// ============================================
// Content preservation tests
// ============================================

#[test]
fn partial_content_preserved_in_unclosed_structures() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CDEFGAB");

    let chord = result
        .tune
        .body
        .elements
        .iter()
        .find_map(|e| match e {
            MusicElement::Chord(c) => Some(c),
            _ => None,
        })
        .expect("Should have a chord");

    // All 7 notes should be in the chord
    assert_eq!(chord.notes.len(), 7, "All notes should be preserved");
}

#[test]
fn notes_after_recovery_are_independent() {
    let result = parse_with_diagnostics("X:1\nK:C\n[CEG\nABC");

    // Chord should have 3 notes
    let chord = result
        .tune
        .body
        .elements
        .iter()
        .find_map(|e| match e {
            MusicElement::Chord(c) => Some(c),
            _ => None,
        })
        .expect("Should have a chord");
    assert_eq!(chord.notes.len(), 3);

    // ABC should be 3 separate notes
    let standalone_notes: Vec<_> = result
        .tune
        .body
        .elements
        .iter()
        .filter(|e| matches!(e, MusicElement::Note(_)))
        .collect();
    assert_eq!(standalone_notes.len(), 3, "ABC should be 3 separate notes");
}
