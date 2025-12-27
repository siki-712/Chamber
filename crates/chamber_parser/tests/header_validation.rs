//! Tests for header validation diagnostics

use chamber_diagnostics::DiagnosticCode;
use chamber_parser::parse_with_diagnostics;

// ============================================
// Missing required fields
// ============================================

#[test]
fn missing_reference_number() {
    let result = parse_with_diagnostics("T:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::MissingReferenceNumber));
}

#[test]
fn missing_key_field() {
    let result = parse_with_diagnostics("X:1\nT:Test\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::MissingKeyField));
}

#[test]
fn missing_title_warning() {
    let result = parse_with_diagnostics("X:1\nK:C\nCDEF");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::MissingTitle));
}

#[test]
fn all_required_fields_present() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nCDEF");

    assert!(!result.has_errors());
    assert!(!result.has_warnings());
}

// ============================================
// Empty field values
// ============================================

#[test]
fn empty_reference_number() {
    let result = parse_with_diagnostics("X:\nT:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyReferenceNumber));
}

#[test]
fn empty_reference_number_with_whitespace() {
    let result = parse_with_diagnostics("X:   \nT:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyReferenceNumber));
}

#[test]
fn invalid_reference_number_not_numeric() {
    let result = parse_with_diagnostics("X:abc\nT:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidReferenceNumber),
        "Expected InvalidReferenceNumber, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn invalid_reference_number_negative() {
    let result = parse_with_diagnostics("X:-1\nT:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidReferenceNumber));
}

#[test]
fn empty_title_warning() {
    let result = parse_with_diagnostics("X:1\nT:\nK:C\nCDEF");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyTitle));
}

#[test]
fn empty_title_with_whitespace() {
    let result = parse_with_diagnostics("X:1\nT:   \nK:C\nCDEF");

    assert!(result.has_warnings());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyTitle));
}

// ============================================
// Field order
// ============================================

#[test]
fn invalid_field_order_title_first() {
    let result = parse_with_diagnostics("T:Test\nX:1\nK:C\nCDEF");

    // Should have both InvalidFieldOrder warning and MissingReferenceNumber error
    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::InvalidFieldOrder),
        "Expected InvalidFieldOrder, got: {:?}",
        codes
    );
}

#[test]
fn valid_field_order() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:4/4\nK:C\nCDEF");

    // X: first is correct order
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidFieldOrder));
}

// ============================================
// Multiple titles (valid)
// ============================================

#[test]
fn multiple_titles_valid() {
    let result = parse_with_diagnostics("X:1\nT:Main Title\nT:Subtitle\nK:C\nCDEF");

    // Multiple T: fields are valid per ABC spec
    assert!(!result.has_errors());
    assert!(!result.has_warnings());
}

// ============================================
// Edge cases
// ============================================

#[test]
fn empty_file() {
    let result = parse_with_diagnostics("");

    // Should report missing X: and K:
    let codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&DiagnosticCode::MissingReferenceNumber));
    assert!(codes.contains(&DiagnosticCode::MissingKeyField));
}

#[test]
fn only_body_no_header() {
    let result = parse_with_diagnostics("CDEF");

    // Should report missing X: and K: (and T: warning)
    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::MissingReferenceNumber));
}

#[test]
fn reference_number_with_leading_zeros() {
    // "01" should parse as valid number 1
    let result = parse_with_diagnostics("X:01\nT:Test\nK:C\nCDEF");

    assert!(!result.has_errors());
}

#[test]
fn reference_number_large() {
    let result = parse_with_diagnostics("X:9999\nT:Test\nK:C\nCDEF");

    assert!(!result.has_errors());
}

// ============================================
// H003: Duplicate reference number
// ============================================

#[test]
fn duplicate_reference_number() {
    let result = parse_with_diagnostics("X:1\nX:2\nT:Test\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::DuplicateReferenceNumber));
}

#[test]
fn multiple_duplicate_reference_numbers() {
    let result = parse_with_diagnostics("X:1\nX:2\nX:3\nT:Test\nK:C\nCDEF");

    // Should have 2 duplicate errors (for X:2 and X:3)
    let dup_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::DuplicateReferenceNumber)
        .count();
    assert_eq!(dup_count, 2, "Expected 2 duplicate errors, got {}", dup_count);
}

// ============================================
// H005: Meter validation
// ============================================

#[test]
fn valid_meter_fraction() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:4/4\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_meter_common_time() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:C\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_meter_cut_time() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:C|\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_meter_none() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:none\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_meter_various() {
    for meter in ["3/4", "6/8", "2/4", "9/8", "12/8", "5/4", "7/8"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nM:{}\nK:C\nCDEF", meter));
        assert!(
            !result.has_errors(),
            "M:{} should be valid, got: {:?}",
            meter,
            result.diagnostics
        );
    }
}

#[test]
fn invalid_meter_text() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:allegro\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidMeterValue),
        "Expected InvalidMeterValue, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn invalid_meter_missing_denominator() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:4/\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidMeterValue));
}

#[test]
fn invalid_meter_non_numeric() {
    let result = parse_with_diagnostics("X:1\nT:Test\nM:a/b\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidMeterValue));
}

// ============================================
// H006: Tempo validation
// ============================================

#[test]
fn valid_tempo_bpm_only() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:120\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_tempo_note_bpm() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:1/4=120\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_tempo_with_name() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:\"Allegro\" 1/4=120\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn valid_tempo_name_only() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:\"Allegro\"\nK:C\nCDEF");
    assert!(!result.has_errors());
}

#[test]
fn invalid_tempo_text() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:fast\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidTempo),
        "Expected InvalidTempo, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn invalid_tempo_bad_fraction() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:a/b=120\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidTempo));
}

#[test]
fn invalid_tempo_bad_bpm() {
    let result = parse_with_diagnostics("X:1\nT:Test\nQ:1/4=fast\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidTempo));
}

// ============================================
// H007: Unit note length validation
// ============================================

#[test]
fn valid_unit_note_length() {
    for length in ["1/4", "1/8", "1/16", "1/2", "1/1"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nL:{}\nK:C\nCDEF", length));
        assert!(
            !result.has_errors(),
            "L:{} should be valid, got: {:?}",
            length,
            result.diagnostics
        );
    }
}

#[test]
fn invalid_unit_note_length_no_slash() {
    let result = parse_with_diagnostics("X:1\nT:Test\nL:4\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidUnitNoteLength),
        "Expected InvalidUnitNoteLength, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn invalid_unit_note_length_text() {
    let result = parse_with_diagnostics("X:1\nT:Test\nL:quarter\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidUnitNoteLength));
}

#[test]
fn invalid_unit_note_length_bad_fraction() {
    let result = parse_with_diagnostics("X:1\nT:Test\nL:a/b\nK:C\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidUnitNoteLength));
}

// ============================================
// H008: Key signature validation
// ============================================

#[test]
fn valid_key_major() {
    for key in ["C", "G", "D", "A", "E", "B", "F"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nK:{}\nCDEF", key));
        assert!(
            !result.has_errors(),
            "K:{} should be valid, got: {:?}",
            key,
            result.diagnostics
        );
    }
}

#[test]
fn valid_key_minor() {
    for key in ["Am", "Em", "Dm", "Gm", "Cm", "Fm", "Bm"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nK:{}\nCDEF", key));
        assert!(
            !result.has_errors(),
            "K:{} should be valid, got: {:?}",
            key,
            result.diagnostics
        );
    }
}

#[test]
fn valid_key_with_accidentals() {
    for key in ["F#", "Bb", "C#m", "Ebm", "F#m", "Bbm"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nK:{}\nCDEF", key));
        assert!(
            !result.has_errors(),
            "K:{} should be valid, got: {:?}",
            key,
            result.diagnostics
        );
    }
}

#[test]
fn valid_key_modes() {
    for key in ["Dmix", "Edor", "Aphr", "Flyd", "Gloc", "Ddorian", "Gmixolydian"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nK:{}\nCDEF", key));
        assert!(
            !result.has_errors(),
            "K:{} should be valid, got: {:?}",
            key,
            result.diagnostics
        );
    }
}

#[test]
fn valid_key_special() {
    for key in ["HP", "Hp", "none"] {
        let result = parse_with_diagnostics(&format!("X:1\nT:Test\nK:{}\nCDEF", key));
        assert!(
            !result.has_errors(),
            "K:{} should be valid, got: {:?}",
            key,
            result.diagnostics
        );
    }
}

#[test]
fn invalid_key_empty() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidKeySignature),
        "Expected InvalidKeySignature, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn invalid_key_lowercase_tonic() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:c\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidKeySignature));
}

#[test]
fn invalid_key_not_a_note() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:H\nCDEF");

    assert!(result.has_errors());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::InvalidKeySignature));
}

#[test]
fn invalid_key_bad_mode() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:Cfoo\nCDEF");

    assert!(result.has_errors());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidKeySignature),
        "Expected InvalidKeySignature for bad mode, got: {:?}",
        result.diagnostics
    );
}

// ============================================
// S001: Empty tune
// ============================================

#[test]
fn empty_tune_warning() {
    let result = parse_with_diagnostics("");

    assert!(result.has_warnings());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyTune),
        "Expected EmptyTune warning, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn whitespace_only_is_empty_tune() {
    let result = parse_with_diagnostics("   \n\n   ");

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::EmptyTune));
}

// ============================================
// S002: Unexpected token
// ============================================

#[test]
fn field_in_body_unexpected() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nCDEF\nM:4/4\nGAB");

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == DiagnosticCode::UnexpectedToken));
}

#[test]
fn title_in_body_unexpected() {
    let result = parse_with_diagnostics("X:1\nT:Test\nK:C\nCDEF\nT:Another Title\nGAB");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnexpectedToken),
        "Expected UnexpectedToken, got: {:?}",
        result.diagnostics
    );
}
