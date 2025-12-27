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
